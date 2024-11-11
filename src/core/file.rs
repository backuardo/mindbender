use crate::error::ApplicationError;
use std::fs;
use std::path::Path;

/// Validate that a file path is valid
pub fn validate_path(file_path: &str) -> Result<(), ApplicationError> {
    match fs::metadata(file_path) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(ApplicationError::InvalidPathError(format!(
            "Path '{}' is not a file.",
            file_path
        ))),
        Err(e) => Err(ApplicationError::IoError(e)),
    }
}

// TODO: this should support reading from stdin
/// Read text data from the specified file path
pub fn read_text(file_path: &str) -> Result<String, ApplicationError> {
    fs::read_to_string(file_path).map_err(ApplicationError::IoError)
}

// TODO: this should support printing to stdout
/// Write text data to the specified file path
pub fn write_text(text: &str, file_path: &str) -> Result<(), ApplicationError> {
    ensure_parent_directory(file_path)?;
    fs::write(file_path, text).map_err(ApplicationError::IoError)
}

/// Ensures that the parent directory exists by creating it if it doesn't
pub fn ensure_parent_directory(file_path: &str) -> Result<(), ApplicationError> {
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent).map_err(ApplicationError::IoError)?;
    }
    Ok(())
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
    fn test_read_text() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = "Hello, world!";
        fs::write(&file_path, content).expect("Failed to write to test file");
        let result = read_text(file_path.to_str().unwrap()).unwrap();

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
    fn test_write_text() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output_text.txt");
        let content = "Test text content";
        let result = write_text(content, file_path.to_str().unwrap());

        assert!(result.is_ok());

        let read_content = fs::read_to_string(file_path).expect("Failed to read written text file");

        assert_eq!(read_content, content);
    }
}
