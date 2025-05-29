use clap::{Parser, ValueEnum};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::env;
use std::io::{self, BufReader, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

/// Represents a single unit file entry for output.
#[derive(Debug, Serialize, Clone)]
struct UnitFileEntry {
    unit_file: String,
    state: String,        // e.g., "enabled", "disabled", "static"
    vendor_preset: String, // e.g., "enabled", "disabled"
}

/// Enum for output format.
#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// Formatted as a table (default).
    Table,
    /// Formatted as JSON array.
    Json,
}

/// Arguments for the 'list' command.
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter unit files by a tag in their [X-extra] section.
    #[arg(long)]
    filter: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,

    /// Do not pipe output into a pager.
    #[arg(long)]
    no_pager: bool,
}

/// Simple INI-like parser for unit files to extract sections and properties.
fn parse_unit_file(path: &Path) -> Result<HashMap<String, HashMap<String, Vec<String>>>, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("Failed to open unit file {}: {}", path.display(), e))?;

    let reader = BufReader::new(file);
    let mut sections: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    let mut current_section: Option<String> = None;

    for line_result in reader.lines() {
        let line = line_result
            .map_err(|e| format!("Failed to read line from {}: {}", path.display(), e))?;
        let line = line.trim();

        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].to_string();
            sections.insert(section_name.clone(), HashMap::new());
            current_section = Some(section_name);
        } else if let Some(section_name) = &current_section {
            if let Some((key, value)) = line.split_once('=') {
                let section_map = sections.get_mut(section_name).unwrap();
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                section_map.entry(key).or_insert_with(Vec::new).push(value);
            }
        }
    }
    Ok(sections)
}

/// Executes the 'list' command logic.
pub fn execute(args: &ListArgs) {
    let mock_units_dir = PathBuf::from("./mock_systemd_units");
    if !mock_units_dir.exists() {
        eprintln!("Error: Mock unit files directory not found: {}", mock_units_dir.display());
        eprintln!("Please create it and add some sample .service files.");
        return;
    }

    let mut unit_files: Vec<UnitFileEntry> = Vec::new();

    for entry in WalkDir::new(&mock_units_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "service")) // Only .service files for now
    {
        let path = entry.path();
        let unit_file_name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.service")
            .to_string();

        let mut matches_filter = true;
        if let Some(filter_value) = &args.filter {
            matches_filter = false; // Assume no match until proven
            if let Ok(sections) = parse_unit_file(path) {
                if let Some(x_extra_section) = sections.get("X-extra") {
                    if let Some(tags) = x_extra_section.get("Tag") {
                        if tags.iter().any(|tag| tag == filter_value) {
                            matches_filter = true;
                        }
                    }
                }
            } else {
                // If parsing fails, treat as no match for filter
                matches_filter = false;
            }
        }

        if matches_filter {
            // Simplistic mock for state and vendor_preset
            // In a real systemctl, this would be complex (symlink checks, actual systemd calls)
            let state = "enabled".to_string();
            let vendor_preset = "enabled".to_string();

            unit_files.push(UnitFileEntry {
                unit_file: unit_file_name,
                state,
                vendor_preset,
            });
        }
    }

    // Sort by unit file name for consistent output
    unit_files.sort_by(|a, b| a.unit_file.cmp(&b.unit_file));

    let output_string = match args.output {
        OutputFormat::Table => {
            let mut s = String::new();
            s.push_str("UNIT FILE                                      STATE           VENDOR PRESET\n");
            s.push_str("--------------------------------------------------------------------------------\n");
            for entry in &unit_files {
                s.push_str(&format!(
                    "{:<45} {:<15} {:<15}\n",
                    entry.unit_file, entry.state, entry.vendor_preset
                ));
            }
            s
        }
        OutputFormat::Json => {
            serde_json::to_string_pretty(&unit_files).expect("Failed to serialize to JSON")
        }
    };

    // Handle pagination
    if !args.no_pager && io::stdout().is_terminal() {
        let pager = env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        let mut command = Command::new(&pager);
        if pager == "less" {
            command.arg("-R"); // -R for raw control characters (colors), though not used here
        }

        match command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
            Ok(mut child) => {
                if let Some(stdin) = child.stdin.as_mut() {
                    if let Err(e) = stdin.write_all(output_string.as_bytes()) {
                        eprintln!("Error writing to pager stdin: {}", e);
                    }
                }
                let _ = child.wait(); // Wait for the pager to exit
            }
            Err(e) => {
                eprintln!("Failed to spawn pager '{}': {}. Printing directly.", pager, e);
                print!("{}", output_string);
            }
        }
    } else {
        print!("{}", output_string);
    }
}
