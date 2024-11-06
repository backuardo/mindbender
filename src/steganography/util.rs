use image::RgbImage;

/// Checks if an image has sufficient capacity to store the given text using LSB steganography.
pub fn is_sufficient_capacity(text: &str, image: &RgbImage) -> bool {
    const BITS_PER_CHAR: usize = 8;
    const DELIMITER_SIZE: usize = 1;

    let text_length = text.len() + DELIMITER_SIZE;
    let total_bits_needed = text_length * BITS_PER_CHAR;
    let available_bits = image.as_flat_samples().samples.len();

    total_bits_needed <= available_bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn create_test_image(width: u32, height: u32) -> RgbImage {
        RgbImage::from_pixel(width, height, Rgb([0, 0, 0]))
    }

    #[test]
    fn test_exact_capacity() {
        let image = create_test_image(2, 2);
        let text = "A";

        assert!(!is_sufficient_capacity(text, &image));
    }

    #[test]
    fn test_sufficient_capacity() {
        let image = create_test_image(10, 10);
        let text = "Hello!";

        assert!(is_sufficient_capacity(text, &image));
    }

    #[test]
    fn test_insufficient_capacity() {
        let image = create_test_image(2, 2);
        let text = "Too long for this image size";

        assert!(!is_sufficient_capacity(text, &image));
    }

    #[test]
    fn test_unicode() {
        let image = create_test_image(5, 5);
        let text = "🦀";

        assert!(is_sufficient_capacity(text, &image));
    }

    #[test]
    fn test_edge_case_single_pixel() {
        let image = create_test_image(1, 1);
        let text = "A";

        assert!(!is_sufficient_capacity(text, &image));
    }
}
