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
        None => handle_tui_mode(),                 // @todo no args present => TUI
        Some(command) => handle_cli_mode(command), // Args present => CLI
    }
}

// @todo launch tui
fn handle_tui_mode() -> Result<(), ApplicationError> {
    todo!()
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
        Commands::GenerateKey { length, output } => {
            let key = cryptography::util::generate_key(length)?;
            match output {
                Some(path) => core::file::write_text(&key, path.to_str().unwrap())?,
                None => println!("Generated key: {}", key),
            }
            Ok(())
        }
    }
}
