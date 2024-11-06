use super::file::{ensure_parent_directory, validate_path};
use crate::error::ApplicationError;
use image::{ImageFormat, ImageReader, RgbImage};
use std::path::Path;

/// Validate that the file path has a supported image extension
pub fn has_valid_image_extension(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "gif" => true,
            _ => false,
        })
        .unwrap_or(false)
}

/// Determine whether a file is lossless
pub fn is_lossless(file_path: &str) -> Result<bool, ApplicationError> {
    let format = ImageFormat::from_path(file_path)
        .map_err(|_| ApplicationError::InvalidPathError("Unsupported image format".to_string()))?;

    match format {
        ImageFormat::Png | ImageFormat::Bmp | ImageFormat::Tiff => Ok(true),
        ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP => Ok(false),
        _ => Err(ApplicationError::InvalidPathError(format!(
            "Unsupported file type '{:?}'",
            format
        ))),
    }
}

/// Convert a lossy image to a lossless format (PNG)
pub fn convert_to_lossless(
    file_path: &str,
    output_path: &str,
) -> Result<RgbImage, ApplicationError> {
    ensure_parent_directory(output_path)?;
    let image = load_image(file_path)?;
    image
        .save_with_format(output_path, ImageFormat::Png)
        .map_err(ApplicationError::ImageError)?;

    Ok(image)
}

/// Load an image and convert it to RgbImage format
pub fn load_image(file_path: &str) -> Result<RgbImage, ApplicationError> {
    validate_path(file_path)?;
    let image_reader = ImageReader::open(file_path)?;
    let image = image_reader.decode()?.to_rgb8();

    Ok(image)
}

/// Write image data to the specified file path
pub fn write_image_file(image: &RgbImage, file_path: &str) -> Result<(), ApplicationError> {
    ensure_parent_directory(file_path)?;

    let format = ImageFormat::from_path(file_path)?;
    image
        .save_with_format(file_path, format)
        .map_err(ApplicationError::ImageError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_validate_path_valid() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        File::create(&file_path).expect("Failed to create test file");
        let result = validate_path(file_path.to_str().unwrap());

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_invalid() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent_file.txt");
        let result = validate_path(file_path.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_is_lossless_png() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_image.png");
        let image = RgbImage::new(10, 10);
        image.save(&file_path).expect("Failed to save image");
        let result = is_lossless(file_path.to_str().unwrap());

        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_lossless_jpeg() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_image.jpg");
        let image = RgbImage::new(10, 10);
        image
            .save_with_format(&file_path, ImageFormat::Jpeg)
            .expect("Failed to save image");
        let result = is_lossless(file_path.to_str().unwrap());

        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_convert_to_lossless() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test_image.jpg");
        let output_path = dir.path().join("converted_image.png");
        let image = RgbImage::new(10, 10);
        image
            .save_with_format(&input_path, ImageFormat::Jpeg)
            .expect("Failed to save image");
        let converted_image =
            convert_to_lossless(input_path.to_str().unwrap(), output_path.to_str().unwrap())
                .expect("Conversion failed");

        assert!(output_path.exists());
        assert_eq!(converted_image.dimensions(), (10, 10));

        let result = is_lossless(output_path.to_str().unwrap());

        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_load_image() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_image.png");
        let image = RgbImage::new(10, 10);
        image.save(&file_path).expect("Failed to save image");
        let loaded_image = load_image(file_path.to_str().unwrap()).expect("Failed to load image");

        assert_eq!(loaded_image.dimensions(), (10, 10));
    }

    #[test]
    fn test_ensure_parent_directory() {
        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("nested").join("file.txt");
        let result = ensure_parent_directory(nested_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_write_image_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output_image.png");
        let image = RgbImage::new(10, 10);
        let result = write_image_file(&image, file_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(file_path.exists());
    }
}
