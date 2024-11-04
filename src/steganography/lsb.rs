use crate::error::ApplicationError;
use image::{Pixel, RgbImage};

use super::util::is_sufficient_capacity;

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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    /// Utility function to create a blank RgbImage of specified dimensions
    fn create_blank_image(width: u32, height: u32) -> RgbImage {
        RgbImage::from_pixel(width, height, Rgb([0, 0, 0]))
    }

    #[test]
    fn test_encode_decode() {
        let image = create_blank_image(10, 10);
        let data = "Hello, World!";

        // Encode data
        let encoded_image = encode(data, &image).expect("Encoding failed");

        // Decode data
        let decoded_data = decode(&encoded_image).expect("Decoding failed");

        // Ensure decoded data matches the original
        assert_eq!(data, decoded_data);
    }

    #[test]
    fn test_insufficient_capacity() {
        let image = create_blank_image(1, 1); // Small image with insufficient capacity
        let data = "This message is too long to fit";

        // Attempt to encode data
        let result = encode(data, &image);

        // Ensure an encoding error is returned
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Encoding error: Image too small to encode data"
        );
    }

    #[test]
    fn test_encode_empty_string() {
        let image = create_blank_image(5, 5);
        let data = "";

        // Encode data
        let encoded_image = encode(data, &image).expect("Encoding failed");

        // Decode data
        let decoded_data = decode(&encoded_image).expect("Decoding failed");

        // Ensure decoded data matches the original (empty string)
        assert_eq!(data, decoded_data);
    }

    #[test]
    fn test_encode_decode_with_delimiter() {
        let image = create_blank_image(10, 10);
        let data = "Message with delimiter test";

        // Encode data
        let encoded_image = encode(data, &image).expect("Encoding failed");

        // Decode data
        let decoded_data = decode(&encoded_image).expect("Decoding failed");

        // Ensure decoded data matches the original, including delimiter handling
        assert_eq!(data, decoded_data);
    }
}
