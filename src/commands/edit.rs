use clap::Parser;
use std::env;
use std::fs;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Arguments for the 'edit' command.
#[derive(Parser, Debug)]
pub struct EditArgs {
    /// The name of the unit (e.g., 'nginx.service') or
    /// the path to a generated unit file if --trans is used.
    pub target: String,
    /// The line number to go to (optional).
    #[arg(short, long)] // -l or --line
    pub line: Option<usize>,
    /// Indicates that 'target' is a generated unit file and its 'SourcePath' should be used.
    #[arg(long)]
    pub trans: bool,
}

/// Extracts the SourcePath from a unit-like configuration file.
/// Assumes a basic INI-like format with a [Unit] section and SourcePath=key.
fn get_source_path(generated_unit_file_path: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(generated_unit_file_path)
        .map_err(|e| format!("Failed to open generated unit file {}: {}", generated_unit_file_path.display(), e))?;

    let reader = BufReader::new(file);
    let mut in_unit_section = false;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line from {}: {}", generated_unit_file_path.display(), e))?;
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            in_unit_section = line == "[Unit]";
        } else if in_unit_section {
            if let Some(rest) = line.strip_prefix("SourcePath=") {
                return Ok(PathBuf::from(rest.trim()));
            }
        }
    }
    Err(format!("SourcePath not found in [Unit] section of {}", generated_unit_file_path.display()))
}

/// Executes the 'edit' command logic.
pub fn execute(args: &EditArgs) {
    let drop_in_file_path: PathBuf;

    // Determine the base name for the drop-in directory based on '--trans' flag
    let unit_name_for_drop_in_dir: String;

    if args.trans {
        let generated_unit_path = PathBuf::from(&args.target);
        println!("--trans flag detected. Parsing {} for SourcePath...", generated_unit_path.display());

        match get_source_path(&generated_unit_path) {
            Ok(source_path) => {
                println!("Found SourcePath: {}", source_path.display());
                // Use the filename of the resolved SourcePath for the drop-in directory
                unit_name_for_drop_in_dir = source_path.file_name()
                                                        .and_then(|s| s.to_str())
                                                        .map(|s| s.to_string())
                                                        .unwrap_or_else(|| {
                                                            eprintln!("Warning: Could not get filename from SourcePath: {}", source_path.display());
                                                            "unknown_source".to_string()
                                                        });
            },
            Err(e) => {
                eprintln!("Error processing generated unit file: {}", e);
                return;
            }
        }
    } else {
        // Default behavior: target is just the unit name
        unit_name_for_drop_in_dir = args.target.clone();
    }

    // --- Construct the drop-in file path ---
    // All drop-ins will be stored in our mock base directory for consistency.
    let base_config_dir = PathBuf::from(".mycli_units"); // Mock /etc/systemd/system/ or ~/.config/systemd/user/

    let unit_drop_in_dir = base_config_dir.join(format!("{}.d", unit_name_for_drop_in_dir));
    drop_in_file_path = unit_drop_in_dir.join("override.conf");

    println!("Attempting to edit drop-in file: {}", drop_in_file_path.display());

    // --- Create directory if it doesn't exist ---
    if !unit_drop_in_dir.exists() {
        if let Err(e) = fs::create_dir_all(&unit_drop_in_dir) {
            eprintln!("Error creating drop-in directory {}: {}", unit_drop_in_dir.display(), e);
            return;
        }
        println!("Created directory: {}", unit_drop_in_dir.display());
    }

    // --- Determine which editor to use ---
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            { "notepad.exe".to_string() }
            #[cfg(not(target_os = "windows"))]
            { "vim".to_string() }
        });

    // --- Launch the editor ---
    let mut command = Command::new(&editor);
    command.arg(&drop_in_file_path);

    // Add line argument if provided (editor-specific, works for vim/nvim/some others)
    if let Some(l) = args.line {
        if editor == "vim" || editor == "nvim" || editor == "vi" {
            command.arg(format!("+{}", l));
        } else if editor == "code" { // VS Code
            command.arg(format!("--goto={}:{}", drop_in_file_path.display(), l));
        }
        // Add more editor-specific logic here if needed
        println!("Attempting to open at line: {}", l);
    }

    match command.status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Editor command exited with non-zero status: {:?}", status.code());
            }
        }
        Err(e) => {
            eprintln!("Failed to execute editor '{}': {}. Make sure it's in your PATH.", editor, e);
        }
    }
}

// This module provides the 'edit' command functionality for a CLI tool.
// It allows users to edit systemd unit files or generated unit files with an optional line number.
// The command supports both direct unit names and generated unit files, extracting the SourcePath when necessary.
// It handles creating necessary directories and launching the user's preferred text editor.
// The code is designed to be flexible and user-friendly, providing clear error messages and feedback.

