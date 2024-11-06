mod cli;
mod cryptography;
mod error;
mod io;
mod steganography;

use crate::cryptography::aes;
use crate::steganography::lsb;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use error::ApplicationError;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ApplicationError> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Encode {
            data_path,
            carrier_path,
            output_path,
            key,
        } => {
            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            pb.enable_steady_tick(Duration::from_millis(80));

            // Load carrier image
            pb.set_message(
                "Loading carrier image..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let mut image = if io::is_lossless(&carrier_path)? {
                io::load_image(&carrier_path)?
            } else {
                println!(
                    "{}",
                    "Warning: Carrier image is lossy. Converting to lossless format...".yellow()
                );
                // Convert to lossless format (PNG) and load the image
                let temp_output = format!("{}.png", output_path);
                io::convert_to_lossless(&carrier_path, &temp_output)?;
                io::load_image(&temp_output)?
            };
            pb.inc(30);

            // Read data from specified file
            pb.set_message(
                "Reading data file..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let data = io::read_text_file(&data_path)?;
            pb.inc(20);

            // Encrypt data if key provided
            pb.set_message(
                "Processing data..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let data = if let Some(key) = key {
                let key_bytes = cryptography::util::key_to_bytes(&key)?;
                aes::encrypt(&data, &key_bytes)?
            } else {
                data
            };
            pb.inc(20);

            // Encode data into the image
            pb.set_message(
                "Encoding data into image..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            lsb::encode(&data, &mut image)?;
            pb.inc(20);

            // Ensure output path has a valid image extension
            pb.set_message(
                "Saving encoded image..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let output_path = if !io::has_valid_image_extension(&output_path) {
                // If not, default to PNG
                format!("{}.png", output_path)
            } else {
                output_path
            };
            // Write encoded image to specified output path
            io::write_image_file(&image, &output_path)?;
            pb.finish_with_message(
                "Encoding completed successfully."
                    .bold()
                    .green()
                    .to_string(),
            );
        }
        Commands::Decode {
            carrier_path,
            output_path,
            key,
        } => {
            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            pb.enable_steady_tick(Duration::from_millis(80));

            // Load the carrier image with encoded data
            pb.set_message(
                "Loading carrier image..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let image = io::load_image(&carrier_path)?;
            pb.inc(30);

            // Decode message from image
            pb.set_message(
                "Decoding data from image..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            let mut decoded_message = lsb::decode(&image)?;
            pb.inc(40);

            // Decrypt message if key provided
            pb.set_message(
                "Processing data..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            if let Some(key) = key {
                // Convert key to 32-byte array
                let key_bytes = cryptography::util::key_to_bytes(&key)?;
                decoded_message = aes::decrypt(&decoded_message, &key_bytes)?;
            }
            pb.inc(20);

            // Write decoded message to specified output path
            pb.set_message(
                "Saving decoded message..."
                    .bright_green()
                    .bold()
                    .italic()
                    .to_string(),
            );
            io::write_text_file(&decoded_message, &output_path)?;
            pb.finish_with_message(
                "Decoding completed successfully."
                    .green()
                    .bold()
                    .to_string(),
            );
        }
    }

    Ok(())
}
