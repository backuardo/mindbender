use crate::error::ApplicationError;
use image::{Pixel, RgbImage};

use super::util::{image_capacity_bits, is_sufficient_capacity};

/// Store text data in the least significant bits of an image's RGB channels
pub fn encode(data: &str, image: &RgbImage) -> Result<RgbImage, ApplicationError> {
    // Append delimiter to the data
    let data_with_delimiter = format!("{}{}", data, '\0');

    // Ensure there is enough capacity to encode the data
    if !is_sufficient_capacity(&data_with_delimiter, image) {
        return Err(ApplicationError::EncodingError(
            "Image too small to encode data".to_string(),
        ));
    }

    // Convert data to bits
    let bits: Vec<u8> = data_with_delimiter
        .bytes()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .collect();

    // Ensure there is enough capacity to encode the data
    // if bits.len() > image_capacity_bits(image) {
    //     return Err(ApplicationError::EncodingError(
    //         "Image too small to encode data".to_string(),
    //     ));
    // }

    // Encode each bit into the image, one channel per pixel
    let mut encoded_image = image.clone();
    let width = image.width() as usize; // Convert width to usize for indexing

    for (i, bit) in bits.iter().enumerate() {
        let pixel_index = i / 3; // Each pixel has 3 channels (RGB)
        let channel_index = i % 3; // 0, 1, or 2 for R, G, B

        let x = (pixel_index % width) as u32;
        let y = (pixel_index / width) as u32;

        let pixel = encoded_image.get_pixel_mut(x, y);
        pixel[channel_index] = (pixel[channel_index] & !1) | bit;
    }

    Ok(encoded_image)
}

/// Extract text data from the least significant bits of an image's RGB channels
pub fn decode(image: &RgbImage) -> Result<String, ApplicationError> {
    // Collect bits from each pixel's least significant bit
    let bits: Vec<u8> = image
        .pixels()
        .flat_map(|pixel| pixel.channels().iter().map(|&channel| channel & 1))
        .collect();

    // Convert bits to bytes, stopping at the delimiter
    let mut bytes: Vec<u8> = Vec::new();
    for chunk in bits.chunks(8) {
        let byte = chunk.iter().fold(0, |acc, &bit| (acc << 1) | bit);
        if byte == 0 {
            // Stop decoding when we hit the null delimiter '\0'
            break;
        }
        bytes.push(byte);
    }

    // Convert bytes to a string
    let message = String::from_utf8(bytes)
        .map_err(|_| ApplicationError::DecodingError("Failed to decode message".to_string()))?;

    Ok(message)
}
