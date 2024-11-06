use image::RgbImage;

pub fn is_sufficient_capacity(text: &str, image: &RgbImage) -> bool {
    // Include the delimiter in the data length
    let data_with_delimiter = format!("{}{}", text, '\0');
    let total_bits_needed = data_with_delimiter.len() * 8; // Each character is 8 bits

    // Get the total number of bits available in the image
    let image_capacity_bits = image.as_flat_samples().samples.len(); // Total number of bytes in the image data

    total_bits_needed <= image_capacity_bits
}
