mod cli;
mod cryptography;
mod error;
mod io;
mod stenography;
mod ui;

use crate::stenography::{lsb, util::is_sufficient_capacity};
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use error::ApplicationError;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ApplicationError> {
    ui::display_welcome();

    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Encode {
            data_path,
            carrier_path,
            output_path,
            key,
        } => {
            // Load carrier image
            let image = io::load_image(&carrier_path)?;
            // Read data from specified file
            let data = io::read_text_file(&data_path)?;
            // Encode data into the image
            let encoded_image = lsb::encode(&data, &image)?;
            // Write encoded image to specified output path
            io::write_image_file(&encoded_image, &output_path)?;

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
            // Load the carrier image with encoded data
            let image = io::load_image(&carrier_path)?;
            // Decode message from image
            let decoded_message = lsb::decode(&image)?;
            // Write decoded message to specified output path
            io::write_text_file(&decoded_message, &output_path)?;

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
