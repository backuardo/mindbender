mod cli;
mod core;
mod cryptography;
mod error;
mod steganography;
mod ui;

use clap::Parser;
use cli::Cli;
use colored::*;
use error::ApplicationError;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

/// Parse command-line arguments and delegate to either the CLI or TUI handler
/// based on whether specific commands were provided.
fn run() -> Result<(), ApplicationError> {
    let cli = Cli::parse();

    match cli.command {
        None => handle_tui_mode(),                 // No args present => TUI
        Some(command) => handle_cli_mode(command), // Args present => CLI
    }
}

fn handle_tui_mode() -> Result<(), ApplicationError> {
    ui::tui::launch_tui()
}

fn handle_cli_mode(command: cli::Commands) -> Result<(), ApplicationError> {
    use cli::Commands;
    use ui::cli::progress::ProgressTracker;

    match command {
        Commands::Encode {
            data_path,
            carrier_path,
            output_path,
            key,
        } => {
            let progress = ProgressTracker::new();
            core::operations::encode(&data_path, &carrier_path, &output_path, key, &progress)
        }
        Commands::Decode {
            carrier_path,
            output_path,
            key,
        } => {
            let progress = ProgressTracker::new();
            core::operations::decode(&carrier_path, &output_path, key, &progress)
        }
    }
}
