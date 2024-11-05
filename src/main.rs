mod cli;
mod cryptography;
mod error;
mod io;
mod steganography;
mod ui;

use crate::cryptography::aes;
use crate::steganography::lsb;
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
            let mut image = io::load_image(&carrier_path)?;
            // Read data from specified file
            let mut data = io::read_text_file(&data_path)?;
            // Encrypt data if key provided
            if let Some(key) = key {
                // Convert key to 32-byte array
                let key_bytes = cryptography::util::key_to_bytes(&key)?;
                data = aes::encrypt(&data, &key_bytes)?;
            }
            // Encode data into the image
            let encoded_image = lsb::encode(&data, &mut image)?;
            // Write encoded image to specified output path
            io::write_image_file(&encoded_image, &output_path)?;
        }
        Commands::Decode {
            carrier_path,
            output_path,
            key,
        } => {
            // Load the carrier image with encoded data
            let image = io::load_image(&carrier_path)?;
            // Decode message from image
            let mut decoded_message = lsb::decode(&image)?;
            // Decrypt message if key provided
            if let Some(key) = key {
                // Convert key to 32-byte array
                let key_bytes = cryptography::util::key_to_bytes(&key)?;
                decoded_message = aes::decrypt(&decoded_message, &key_bytes)?;
            }
            // Write decoded message to specified output path
            io::write_text_file(&decoded_message, &output_path)?;
        }
    }

    Ok(())
}
