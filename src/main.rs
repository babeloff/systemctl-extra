use clap::{Parser, Subcommand};

// Declare the 'commands' module, which then declares 'edit'
mod commands;
mod systemd_modes;
use systemd_modes::{HasPrivilege, UnitPrivilege, get_system_unit_paths, get_user_unit_paths, get_local_unit_paths, get_unit_file_extensions};  
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    // Make the command optional. If no subcommand is provided, Clap automatically
    // displays the main help message.
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
pub struct MainArgs {
    /// The privilege of the 
    #[arg(short, long, default_value_t = UnitPrivilege::System)]
    pub location: UnitPrivilege,
}

impl HasPrivilege for MainArgs {
    fn privilege(&self) -> UnitPrivilege {
        self.location.clone()
    }
}


#[derive(Subcommand, Debug)]
enum Commands {
    /// Edit a file, optionally at a specific line.
    Edit(commands::edit::EditArgs),
    /// List unit files with optional filtering and output formats.
    List(commands::list::ListArgs),
}

fn get_unit_search_paths<T: HasPrivilege>(args: T) -> Vec<PathBuf> {
    match args.privilege() {
        UnitPrivilege::System => {
            println!("Searching system unit paths:");
            let paths = get_system_unit_paths();
            for p in &paths { println!(" - {}", p.display()); }
            paths
        }
        UnitPrivilege::User => {
            println!("Searching user unit paths:");
            let paths = get_user_unit_paths();
            for p in &paths { println!(" - {}", p.display()); }
            paths
        }
        UnitPrivilege::Local => {
            println!("Searching local mock unit paths:");
            let paths = get_local_unit_paths();
            for p in &paths { println!(" - {}", p.display()); }
            // Ensure the local mock directory exists for local tests
            if let Some(local_path) = paths.get(0) {
                if !local_path.exists() {
                    eprintln!("Error: Local mock unit files directory not found: {}", local_path.display());
                    eprintln!("Please create it (e.g., `mkdir mock_systemd_units`) and add some sample unit files (e.g., `.service`).");
                }
            }
            paths
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Edit(args)) => { // Match the 'Edit' subcommand
            let _unit_extensions = get_unit_file_extensions();
            let _paths = get_unit_search_paths(args.clone());
            commands::edit::execute(&args);
        }
        Some(Commands::List(args)) => { // Match the 'List' subcommand
            let unit_extensions = get_unit_file_extensions();
            let paths = get_unit_search_paths(args.clone());
            commands::list::execute(paths, unit_extensions, &args);
        }
        None => {
            // If no subcommand is provided, Clap will automatically show help.
            // This 'None' branch won't typically be hit for the default help behavior,
            // but it's good to be aware of if you wanted to implement custom logic.
            // For Clap's default help, just running `cargo run --` is enough.
        }
    }
}