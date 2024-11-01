use crate::error::ApplicationError;
use colored::*;
use image::{ImageFormat, ImageReader, RgbImage};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Validate that a file path is valid
pub fn validate_path(file_path: &str) -> Result<(), ApplicationError> {
    if !fs::metadata(file_path)?.is_file() {
        return Err(ApplicationError::InvalidPathError(format!(
            "Path '{}' is not a file.",
            file_path
        )));
    }
    Ok(())
}

/// Determine whether a file is lossless
fn is_lossless(file_path: &str) -> Result<bool, ApplicationError> {
    let format = ImageFormat::from_path(file_path)?;
    match format {
        ImageFormat::WebP => {
            // Assume lossy; specific library check required for lossless WebP detection
            println!("{}", "Warning: WebP detected; assuming lossy".yellow());
            Ok(false)
        }
        ImageFormat::Png | ImageFormat::Bmp | ImageFormat::Tiff => Ok(true),
        ImageFormat::Jpeg | ImageFormat::Gif => Ok(false),
        _ => Err(ApplicationError::InvalidPathError(format!(
            "Unsupported file type '{:?}'",
            format
        ))),
    }
}

/// Read text data from the specified file path
pub fn read_text_file(file_path: &str) -> Result<String, ApplicationError> {
    let mut file = File::open(file_path).map_err(ApplicationError::IoError)?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(ApplicationError::IoError)?;
    Ok(content)
}

/// Load an image and convert it to RgbImage format
pub fn load_image(file_path: &str) -> Result<RgbImage, ApplicationError> {
    validate_path(file_path)?;

    let image_reader = ImageReader::open(file_path)?;
    let image = image_reader.decode()?.to_rgb8();

    Ok(image)
}

/// Ensures that the parent directory exists by creating it if it doesn't
pub fn ensure_parent_directory(file_path: &str) -> Result<(), ApplicationError> {
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent).map_err(ApplicationError::IoError)?;
    }
    Ok(())
}

/// Write image data to the specified file path
pub fn write_image_file(image: &RgbImage, file_path: &str) -> Result<(), ApplicationError> {
    ensure_parent_directory(file_path)?;

    let format = ImageFormat::from_path(file_path)?;
    image
        .save_with_format(file_path, format)
        .map_err(ApplicationError::ImageError)
}

/// Write text data to the specified file path
pub fn write_text_file(text: &str, file_path: &str) -> Result<(), ApplicationError> {
    ensure_parent_directory(file_path)?;

    let mut file = File::create(file_path).map_err(ApplicationError::IoError)?;
    file.write_all(text.as_bytes())
        .map_err(ApplicationError::IoError)
}
