use crate::error::ApplicationError;
use aes_gcm::aead::{rand_core::RngCore, OsRng};
use base64::{engine::general_purpose, Engine};
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

/// Generate an encryption key
pub fn generate_key(length: Option<usize>) -> Result<String, ApplicationError> {
    let key_length = length.unwrap_or(32);
    let mut key = vec![0u8; key_length];
    OsRng.fill_bytes(&mut key);
    Ok(general_purpose::STANDARD.encode(key))
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

    #[test]
    fn test_generate_key() {
        let key = generate_key(None).unwrap();
        assert_eq!(general_purpose::STANDARD.decode(&key).unwrap().len(), 32);
    }

    #[test]
    fn test_generate_custom_length_key() {
        let length = 16;
        let key = generate_key(Some(length)).unwrap();
        assert_eq!(
            general_purpose::STANDARD.decode(&key).unwrap().len(),
            length
        );
    }
}
