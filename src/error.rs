use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Command-line parsing error: {0}")]
    CliError(#[from] clap::Error),

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
