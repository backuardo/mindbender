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

fn run() -> Result<(), ApplicationError> {
    let cli = Cli::parse();

    match cli.command {
        None => launch_tui(),
        Some(command) => handle_cli(command),
    }
}

fn launch_tui() -> Result<(), ApplicationError> {
    println!("{}", "Launch TUI...".purple());
    // TODO: Call into TUI module
    Ok(())
}

fn handle_cli(command: cli::Commands) -> Result<(), ApplicationError> {
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
