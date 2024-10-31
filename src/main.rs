mod cli;

use crate::cli::{Cli, Commands};
use clap::{Error, Parser};
use colored::*;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
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
