use crate::error::ApplicationError;
use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine};

const NONCE_SIZE: usize = 12;

/// Encrypt plaintext data with a key using AES GCM mode, returning a base64-encoded string
pub fn encrypt(data: &str, key: &[u8; 32]) -> Result<String, ApplicationError> {
    let cipher = Aes256Gcm::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|_| ApplicationError::EncryptionError("Encryption failed".to_string()))?;

    let mut encrypted_data = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    encrypted_data.extend_from_slice(&nonce_bytes);
    encrypted_data.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(encrypted_data))
}

/// Decrypt base64-encoded data with a key using AES GCM mode
pub fn decrypt(encoded_data: &str, key: &[u8; 32]) -> Result<String, ApplicationError> {
    let cipher = Aes256Gcm::new(key.into());

    let encrypted_data = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|e| {
            ApplicationError::DecryptionError(format!("Invalid base64 encoding: {}", e))
        })?;

    if encrypted_data.len() < NONCE_SIZE {
        return Err(ApplicationError::DecryptionError(
            "Encrypted data too short".to_string(),
        ));
    }

    let (nonce, ciphertext) = encrypted_data.split_at(NONCE_SIZE);

    let decrypted_data = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| ApplicationError::DecryptionError(format!("Decryption failed: {}", e)))?;

    String::from_utf8(decrypted_data).map_err(|e| {
        ApplicationError::DecryptionError(format!("Invalid UTF-8 in decrypted data: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let data = "Test message for encryption";
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_decrypt_with_invalid_key() {
        let original_key = [0u8; 32];
        let invalid_key = [1u8; 32];
        let data = "This message will not decrypt properly";
        let encrypted_data = encrypt(data, &original_key).expect("Encryption failed");
        let result = decrypt(&encrypted_data, &invalid_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let key = [0u8; 32];
        let data = "";
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_randomized_keys() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let data = "Testing encryption with a random key";
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        assert_eq!(data, decrypted_data);
    }
}
