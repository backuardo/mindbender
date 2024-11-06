mod cli;
mod core;
mod cryptography;
mod error;
mod steganography;
mod ui;

use crate::cryptography::{aes, util::key_to_bytes};
use crate::steganography::lsb;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use error::ApplicationError;
use ui::ProgressTracker;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ApplicationError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode {
            data_path,
            carrier_path,
            output_path,
            key,
        } => {
            let tracker = ProgressTracker::new();

            // Load carrier image
            tracker.update("Loading carrier image...");
            let mut image = if core::image::is_lossless(&carrier_path)? {
                core::image::load_image(&carrier_path)?
            } else {
                println!(
                    "{}",
                    "Warning: Carrier image is lossy. Converting to lossless format...".yellow()
                );
                let temp_output = format!("{}.png", output_path);
                core::image::convert_to_lossless(&carrier_path, &temp_output)?;
                core::image::load_image(&temp_output)?
            };

            tracker.update("Reading data file...");
            let data = core::file::read_text_file(&data_path)?;

            let data = if let Some(key) = key {
                tracker.update("Encrypting data...");
                let key_bytes = key_to_bytes(&key)?;
                aes::encrypt(&data, &key_bytes)?
            } else {
                data
            };

            // Encode data into the image
            tracker.update("Encoding data into image...");
            lsb::encode(&data, &mut image)?;

            // Save the encoded image
            tracker.update("Saving encoded image...");
            let output_path = if !core::image::has_valid_image_extension(&output_path) {
                format!("{}.png", output_path)
            } else {
                output_path
            };
            core::image::write_image_file(&image, &output_path)?;

            tracker.finish_with_message("Encoding completed successfully.");
        }

        Commands::Decode {
            carrier_path,
            output_path,
            key,
        } => {
            let tracker = ProgressTracker::new();

            // Load the carrier image
            tracker.update("Loading carrier image...");
            let image = core::image::load_image(&carrier_path)?;

            // Decode message
            tracker.update("Decoding data from image...");
            let mut decoded_message = lsb::decode(&image)?;

            // Process decoded data
            if let Some(key) = key {
                tracker.update("Decrypting data...");
                let key_bytes = key_to_bytes(&key)?;
                decoded_message = aes::decrypt(&decoded_message, &key_bytes)?;
            }

            // Save decoded message
            tracker.update("Saving decoded message...");
            core::file::write_text_file(&decoded_message, &output_path)?;

            tracker.finish_with_message("Decoding completed successfully.");
        }
    }

    Ok(())
}
