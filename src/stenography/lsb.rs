use crate::error::ApplicationError;
use image::{Pixel, RgbImage};

use super::util::image_capacity_bits;

/// Store text data in the least significant bits of an image's RGB channels
pub fn encode(data: &str, image: &RgbImage) -> Result<RgbImage, ApplicationError> {
    // Convert data to bits
    let bits: Vec<u8> = data
        .bytes()
        // Create a bit for each position in the byte
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .collect();

    // Ensure there is enough capacity to encode the data
    if bits.len() > image_capacity_bits(image) {
        return Err(ApplicationError::EncodingError(
            "Image too small to encode data".to_string(),
        ));
    }

    // Encode the bits into the image pixels
    let mut encoded_image = image.clone();
    encoded_image
        .pixels_mut()
        .enumerate()
        .for_each(|(i, pixel)| {
            if let Some(bit) = bits.get(i % bits.len()) {
                for channel in pixel.channels_mut() {
                    // Set LSB to the bit
                    *channel = (*channel & !1) | bit;
                }
            }
        });

    Ok(encoded_image)
}

/// Extract text data from the least significant bits of an image's RGB channels
pub fn decode(image: &RgbImage) -> Result<String, ApplicationError> {
    // Collect bits from each pixel's least significant bit
    let bits: Vec<u8> = image
        .pixels()
        .flat_map(|pixel| pixel.channels().iter().map(|&channel| channel & 1))
        .collect();

    // Convert bits to bytes, then to characters
    let bytes: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| chunk.iter().fold(0, |acc, &bit| (acc << 1) | bit))
        .collect();

    let message = String::from_utf8(bytes)
        .map_err(|_| ApplicationError::DecodingError("Failed to decode message".to_string()))?;

    Ok(message)
}
