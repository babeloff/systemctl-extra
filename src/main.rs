use clap::{Parser, Subcommand};

// Declare the 'commands' module, which then declares 'edit'
mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    // Make the command optional. If no subcommand is provided, Clap automatically
    // displays the main help message.
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Edit a file, optionally at a specific line.
    Edit(commands::edit::EditArgs), // 'edit' is now a subcommand again
    // The 'Goodbye' command is removed
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Edit(args)) => { // Match the 'Edit' subcommand
            commands::edit::execute(&args);
        }
        None => {
            // If no subcommand is provided, Clap will automatically show help.
            // This 'None' branch won't typically be hit for the default help behavior,
            // but it's good to be aware of if you wanted to implement custom logic.
            // For Clap's default help, just running `cargo run --` is enough.
        }
    }
}