use crate::error::ApplicationError;
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

/// Read text data from the specified file path
pub fn read_text_file(file_path: &str) -> Result<String, ApplicationError> {
    let mut file = File::open(file_path).map_err(ApplicationError::IoError)?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(ApplicationError::IoError)?;
    Ok(content)
}

/// Ensures that the parent directory exists by creating it if it doesn't
pub fn ensure_parent_directory(file_path: &str) -> Result<(), ApplicationError> {
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent).map_err(ApplicationError::IoError)?;
    }
    Ok(())
}

/// Write text data to the specified file path
pub fn write_text_file(text: &str, file_path: &str) -> Result<(), ApplicationError> {
    ensure_parent_directory(file_path)?;

    let mut file = File::create(file_path).map_err(ApplicationError::IoError)?;
    file.write_all(text.as_bytes())
        .map_err(ApplicationError::IoError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
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
    fn test_read_text_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = "Hello, world!";
        fs::write(&file_path, content).expect("Failed to write to test file");

        let result = read_text_file(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result, content);
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
    fn test_write_text_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output_text.txt");
        let content = "Test text content";

        let result = write_text_file(content, file_path.to_str().unwrap());
        assert!(result.is_ok());

        let read_content = fs::read_to_string(file_path).expect("Failed to read written text file");
        assert_eq!(read_content, content);
    }
}
