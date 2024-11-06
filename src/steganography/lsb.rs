use super::util::is_sufficient_capacity;
use crate::error::ApplicationError;
use image::{Pixel, RgbImage};
use rayon::prelude::*;

const NULL_DELIMITER: char = '\0';
const BITS_PER_BYTE: usize = 8;

/// Encodes text data into an image using LSB (Least Significant Bit) steganography.
///
/// # How it works
/// - Each character is split into 8 bits
/// - Each bit is stored in the least significant bit of an RGB channel
/// - A null character ('\0') is appended as a delimiter
pub fn encode(data: &str, image: &mut RgbImage) -> Result<(), ApplicationError> {
    let data_with_delimiter = format!("{}{}", data, NULL_DELIMITER);

    if !is_sufficient_capacity(&data_with_delimiter, image) {
        return Err(ApplicationError::EncodingError(
            "Image too small to encode data".to_string(),
        ));
    }

    let image_data = image.as_flat_samples_mut().samples;

    image_data
        .par_chunks_mut(BITS_PER_BYTE)
        .zip(data_with_delimiter.as_bytes().par_iter())
        .for_each(|(chunk, &data_byte)| {
            chunk.iter_mut().enumerate().for_each(|(i, pixel_byte)| {
                let bit = (data_byte >> (BITS_PER_BYTE - 1 - i)) & 1;
                *pixel_byte = (*pixel_byte & !1) | bit;
            });
        });

    Ok(())
}

/// Decodes text data from an image that was encoded using LSB steganography.
///
/// # How it works
/// - Collects the least significant bit from each RGB channel
/// - Combines bits into bytes until a null delimiter is found
/// - Converts the bytes back into a UTF-8 string
pub fn decode(image: &RgbImage) -> Result<String, ApplicationError> {
    let mut bits = Vec::with_capacity(image.width() as usize * image.height() as usize * 3);

    image
        .pixels()
        .flat_map(|pixel| pixel.channels().iter())
        .for_each(|&channel| bits.push(channel & 1));

    let mut bytes = Vec::with_capacity(bits.len() / BITS_PER_BYTE);
    for byte_bits in bits.chunks(BITS_PER_BYTE) {
        if byte_bits.len() != BITS_PER_BYTE {
            break;
        }

        let byte = byte_bits.iter().fold(0u8, |acc, &bit| (acc << 1) | bit);

        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }

    String::from_utf8(bytes).map_err(|e| {
        ApplicationError::DecodingError(format!("Invalid UTF-8 sequence in decoded data: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn create_blank_image(width: u32, height: u32) -> RgbImage {
        RgbImage::from_pixel(width, height, Rgb([0, 0, 0]))
    }

    #[test]
    fn test_encode_decode() {
        let mut image = create_blank_image(10, 10);
        let data = "Hello, World!";
        encode(data, &mut image).expect("Encoding failed");
        let decoded_data = decode(&image).expect("Decoding failed");

        assert_eq!(data, decoded_data);
    }

    #[test]
    fn test_insufficient_capacity() {
        let mut image = create_blank_image(1, 1);
        let data = "This message is too long to fit";
        let result = encode(data, &mut image);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Encoding error: Image too small to encode data"
        );
    }

    #[test]
    fn test_encode_empty_string() {
        let mut image = create_blank_image(5, 5);
        let data = "";
        encode(data, &mut image).expect("Encoding failed");
        let decoded_data = decode(&image).expect("Decoding failed");

        assert_eq!(data, decoded_data);
    }

    #[test]
    fn test_encode_decode_with_delimiter() {
        let mut image = create_blank_image(10, 10);
        let data = "Message with delimiter test";
        encode(data, &mut image).expect("Encoding failed");
        let decoded_data = decode(&image).expect("Decoding failed");

        assert_eq!(data, decoded_data);
    }
}
