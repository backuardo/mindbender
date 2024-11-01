use image::RgbImage;

pub fn image_capacity_bits(image: &RgbImage) -> usize {
    (image.width() * image.height() * 3) as usize // 3 channels (RGB)
}
