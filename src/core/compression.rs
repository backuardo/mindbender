use crate::error::ApplicationError;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::prelude::*;

/// Compress data
pub fn compress(data: &[u8]) -> Result<Vec<u8>, ApplicationError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| ApplicationError::IoError(e))?;
    encoder.finish().map_err(|e| ApplicationError::IoError(e))
}

/// Decompress data
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, ApplicationError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ApplicationError::IoError(e))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original_data = b"Hello, world!";
        let compressed_data = compress(original_data).expect("Compression failed");
        let decompressed_data = decompress(&compressed_data).expect("Decompression failed");

        assert_eq!(original_data.to_vec(), decompressed_data);
    }

    #[test]
    fn test_compression_error_handling() {
        let empty_data: &[u8] = &[];
        let compressed_data = compress(empty_data);
        assert!(compressed_data.is_ok());
    }

    #[test]
    fn test_decompression_error_handling() {
        let invalid_data = b"This is not compressed!";
        let decompressed_data = decompress(invalid_data);
        assert!(decompressed_data.is_err());
    }
}
