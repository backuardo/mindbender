use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Invalid path error: {0}")]
    InvalidPathError(String),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),
}
