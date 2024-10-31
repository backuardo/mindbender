mod cli;
mod cryptography;
mod io;
mod stenography;
mod ui;

use crate::cli::{Cli, Commands};
use crate::cryptography::aes::AesCipher;
use crate::stenography::lsb::LsbCodec;
use crate::ui::display_welcome;
use clap::Parser;
use colored::*;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    display_welcome();

    let cli: Cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    match cli.command {
        Commands::Encode {
            data_path,
            carrier_path,
            output_path,
            key,
        } => {
            // Check args
            // Conditionally encrypt data
            // Encode data
            // Write encoded image data
            // --- Debug
            println!("{}", "Encode!".green());
            println!(
                "data_path: {}\ncarrier_path: {}\noutput_path: {}",
                data_path.blue(),
                carrier_path.blue(),
                output_path.blue(),
            );
            if let Some(key) = key {
                println!("key: {}", key.blue())
            }
        }
        Commands::Decode {
            carrier_path,
            output_path,
            key,
        } => {
            // Check args
            // Decode data
            // Decrypt data
            // Write decrypted and decoded data
            // --- Debug
            println!("{}", "Decode!".green());
            println!(
                "carrier_path: {}\noutput_path: {}",
                carrier_path.blue(),
                output_path.blue(),
            );
            if let Some(key) = key {
                println!("key: {}", key.blue())
            }
        }
    }

    Ok(())
}
