use image::RgbImage;

pub fn image_capacity_bits(image: &RgbImage) -> usize {
    (image.width() * image.height() * 3) as usize // 3 channels (RGB)
}

pub fn image_capacity_bytes(image: &RgbImage) -> usize {
    image_capacity_bits(image) / 8
}

pub fn is_sufficient_capacity(text: &str, image: &RgbImage) -> bool {
    text.len() <= image_capacity_bytes(image)
}
