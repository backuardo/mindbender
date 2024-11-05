use crate::error::ApplicationError;
use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine};

/// Encrypt plaintext data with a key using AES GCM mode, returning a base64-encoded string
pub fn encrypt(data: &str, key: &[u8; 32]) -> Result<String, ApplicationError> {
    let cipher = Aes256Gcm::new(key.into());

    // Generate random nonce
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    // Encrypt data
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data.as_bytes())
        .map_err(|_| ApplicationError::EncryptionError("Encryption failed".to_string()))?;

    // Concatenate nonce and ciphertext, then base64 encode
    let mut encrypted_data = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    Ok(general_purpose::STANDARD.encode(encrypted_data))
}

/// Decrypt base64-encoded data with a key using AES GCM mode
pub fn decrypt(encoded_data: &str, key: &[u8; 32]) -> Result<String, ApplicationError> {
    let cipher = Aes256Gcm::new(key.into());

    // Decode the base64-encoded data
    let encrypted_data = general_purpose::STANDARD
        .decode(encoded_data)
        .map_err(|_| {
            ApplicationError::DecryptionError("Failed to decode base64 data".to_string())
        })?;

    // Separate nonce and ciphertext
    let (nonce, ciphertext) = encrypted_data.split_at(12);

    // Decrypt data
    let decrypted_data = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| ApplicationError::DecryptionError("Decryption failed".to_string()))?;

    // Convert decrypted bytes to a string
    String::from_utf8(decrypted_data).map_err(|_| {
        ApplicationError::DecryptionError("Invalid UTF-8 sequence in decrypted data".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32]; // Use a fixed key for testing
        let data = "Test message for encryption";

        // Encrypt the data
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");

        // Decrypt the data
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        // Check that the decrypted data matches the original data
        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_decrypt_with_invalid_key() {
        let original_key = [0u8; 32];
        let invalid_key = [1u8; 32]; // Use a different key to simulate an invalid decryption
        let data = "This message will not decrypt properly";

        // Encrypt the data with the original key
        let encrypted_data = encrypt(data, &original_key).expect("Encryption failed");

        // Attempt to decrypt with the invalid key
        let result = decrypt(&encrypted_data, &invalid_key);

        // Ensure decryption fails
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let key = [0u8; 32]; // Fixed key for testing
        let data = ""; // Empty string

        // Encrypt the empty string
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");

        // Decrypt the data
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        // Check that the decrypted data matches the original (empty string)
        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_randomized_keys() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key); // Generate a random key
        let data = "Testing encryption with a random key";

        // Encrypt the data
        let encrypted_data = encrypt(data, &key).expect("Encryption failed");

        // Decrypt the data
        let decrypted_data = decrypt(&encrypted_data, &key).expect("Decryption failed");

        // Ensure the decrypted data matches the original data
        assert_eq!(data, decrypted_data);
    }
}
