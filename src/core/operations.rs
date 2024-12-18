use crate::core;
use crate::cryptography::{aes, util::key_to_bytes};
use crate::error::ApplicationError;
use crate::steganography::lsb;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;
use colored::*;

/// Progress tracking interface
pub trait Progress {
    fn update(&self, message: &str);
    fn finish_with_message(&self, message: &str);
}

/// Encodes a message into an image using LSB steganography
///
/// 1. Loads and validates the carrier image
/// 2. Converts lossy images to lossless format if necessary
/// 3. Reads the message from the data file
/// 4. Optionally encrypts the message using the provided key
/// 5. Optionally compresses the message
/// 6. Encodes the message into the image using LSB steganography
/// 7. Saves the resulting image to the specified output path
pub fn encode(
    data_path: &str,
    carrier_path: &str,
    output_path: &str,
    key: Option<String>,
    compress: bool,
    progress: &impl Progress,
) -> Result<(), ApplicationError> {
    progress.update("Loading carrier image...");
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

    progress.update("Reading data file...");
    let data = core::file::read_text(&data_path)?;

    let mut data = if let Some(key) = key {
        progress.update("Encrypting data...");
        let key_bytes = key_to_bytes(&key)?;
        aes::encrypt(&data, &key_bytes)?
    } else {
        data
    };

    if compress {
        progress.update("Compressing data...");
        let compressed_data = core::compression::compress(data.as_bytes())?;
        // Add a marker to indicate compression
        data = format!("COMPRESSED:{}", BASE64_ENGINE.encode(&compressed_data));
    }

    progress.update("Encoding data into image...");
    lsb::encode(&data, &mut image)?;

    progress.update("Saving encoded image...");
    let output_path = if !core::image::has_valid_image_extension(&output_path) {
        format!("{}.png", output_path)
    } else {
        output_path.to_string()
    };
    core::image::write_image_file(&image, &output_path)?;

    progress.finish_with_message(&format!(
        "Encoding completed successfully => {}",
        output_path
    ));

    Ok(())
}

/// Decodes a message from an image using LSB steganography
///
/// 1. Loads the carrier image containing the hidden message
/// 2. Extracts the message using LSB steganography
/// 3. Optionally decrypts the message using the provided key
/// 4. Optionally decompresses the message
/// 5. Saves the decoded message to the specified output path
pub fn decode(
    carrier_path: &str,
    output_path: &str,
    key: Option<String>,
    decompress: bool,
    progress: &impl Progress,
) -> Result<(), ApplicationError> {
    progress.update("Loading carrier image...");
    let image = core::image::load_image(&carrier_path)?;

    progress.update("Decoding data from image...");
    let mut decoded_message = lsb::decode(&image)?;

    if let Some(key) = key {
        progress.update("Decrypting data...");
        let key_bytes = key_to_bytes(&key)?;
        decoded_message = aes::decrypt(&decoded_message, &key_bytes)?;
    }

    if decompress {
        progress.update("Decompressing data...");
        if !decoded_message.starts_with("COMPRESSED:") {
            return Err(ApplicationError::DecodingError(
                "Decompression expected, but message is not compressed".to_string(),
            ));
        }
        let base64_data = &decoded_message["COMPRESSED:".len()..];
        let compressed_data = BASE64_ENGINE
            .decode(base64_data)
            .map_err(|_| ApplicationError::DecodingError("Base64 decoding failed".to_string()))?;
        decoded_message = String::from_utf8(core::compression::decompress(&compressed_data)?)
            .map_err(|e| {
                ApplicationError::DecodingError(format!("UTF-8 decoding failed: {}", e))
            })?;
    } else if decoded_message.starts_with("COMPRESSED:") {
        return Err(ApplicationError::DecodingError(
            "Data is compressed but decompression was not requested".to_string(),
        ));
    }

    progress.update("Saving decoded message...");
    core::file::write_text(&decoded_message, &output_path)?;

    progress.finish_with_message(&format!(
        "Decoding completed successfully => {}",
        output_path
    ));

    Ok(())
}
