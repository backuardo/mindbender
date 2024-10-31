# Mindbender

A versatile command-line steganography tool for hiding data within any carrier file.
Securely embed messages, files, or encrypted data into various media formats while preserving the carrier file's functionality.

Key Features:
- Universal carrier file support - hide data in videos, images, documents, and more
- Built-in encryption support for enhanced security
- Simple CLI interface with straightforward commands
- Preserves original file integrity and functionality

## Usage

### Encode a message into a carrier file
`mindbender encode [data_to_hide] [carrier_file] -o [output_file]`

### Decode a carrier file and reveal the hidden data
`mindbender decode [carrier_file] -o [output_file]`
