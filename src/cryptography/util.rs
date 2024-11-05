use crate::error::ApplicationError;

pub fn key_to_bytes(key: &str) -> Result<[u8; 32], ApplicationError> {
    // Convert key string to bytes, padding or truncating to fit 32 bytes
    let mut key_bytes = [0u8; 32];
    let key_as_bytes = key.as_bytes();

    if key_as_bytes.len() > 32 {
        return Err(ApplicationError::EncryptionError(
            "Key must be 32 bytes or fewer".to_string(),
        ));
    }

    // Copy key bytes into 32-byte array, padding with zeros if necessary
    key_bytes[..key_as_bytes.len()].copy_from_slice(key_as_bytes);
    Ok(key_bytes)
}
