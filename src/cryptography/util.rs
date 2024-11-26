use crate::error::ApplicationError;
use colored::*;

const KEY_SIZE: usize = 32;

/// Convert a string key into a fixed 32-byte array for AES-256 encryption
pub fn key_to_bytes(key: &str) -> Result<[u8; 32], ApplicationError> {
    let key_bytes = key.as_bytes();

    if key_bytes.len() > KEY_SIZE {
        return Err(ApplicationError::EncryptionError(format!(
            "Key length {} exceeds maximum of {} bytes",
            key_bytes.len(),
            KEY_SIZE
        )));
    }

    if key_bytes.len() < KEY_SIZE {
        println!("{}", "Warning: insecure key length".yellow());
    }

    let mut result = [0u8; KEY_SIZE];
    result[..key_bytes.len()].copy_from_slice(key_bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_length_key() {
        let key = "12345678901234567890123456789012";
        let result = key_to_bytes(key).unwrap();

        assert_eq!(result.len(), KEY_SIZE);
        assert_eq!(&result, key.as_bytes());
    }

    #[test]
    fn test_short_key() {
        let key = "short-key";
        let result = key_to_bytes(key).unwrap();

        assert_eq!(result.len(), KEY_SIZE);
        assert_eq!(&result[..key.len()], key.as_bytes());
        assert!(result[key.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_empty_key() {
        let key = "";
        let result = key_to_bytes(key).unwrap();

        assert_eq!(result.len(), KEY_SIZE);
        assert!(result.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_too_long_key() {
        let key = "12345678901234567890123456789012X";
        let result = key_to_bytes(key);

        assert!(result.is_err());
        assert!(matches!(result, Err(ApplicationError::EncryptionError(_))));
    }

    #[test]
    fn test_unicode_key() {
        let key = "🔑";
        let result = key_to_bytes(key).unwrap();

        assert_eq!(result.len(), KEY_SIZE);
        assert_eq!(&result[..4], key.as_bytes());
        assert!(result[4..].iter().all(|&b| b == 0));
    }
}
