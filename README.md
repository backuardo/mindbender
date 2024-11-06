
# Mindbender

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

*A steganography CLI tool for encrypting and hiding text within images.*

## Introduction

**MindBender** is a command-line tool that allows you to hide (and retrieve) text data within images using steganography techniques. It supports optional encryption for added security, ensuring your hidden messages remain confidential.

## Features

- **Steganography Encoding and Decoding**: Hide text data within images.
- **Encryption Support**: Optional AES-256 encryption for secure data.
- **Automatic Lossy Image Conversion**: Converts lossy images to lossless formats automatically.
- **User-Friendly Interface**: Clear commands and helpful messages.
- **Cross-Platform Support**: Works on Windows, macOS, and Linux.

## Usage

MindBender provides `encode` and `decode` commands.

### Encode a message
```
mindbender encode [OPTIONS] <DATA_FILE_PATH> <CARRIER_FILE_PATH>
```
- `DATA_FILE_PATH`: Path to the text file containing the message to encode.
- `CARRIER_FILE_PATH`: Path to the image file to use as the carrier.

**Options**
-   `-o`, `--output-path <OUTPUT_FILE_PATH>`: Output path for the encoded image (default: `output.png`).
-   `-k`, `--key <KEY>`: Optional encryption key.

**Example:**
```
mindbender encode secret_message.txt carrier.jpg --output-path hidden.png --key "my_secret_key"
```

### Decode a message
```
mindbender decode [OPTIONS] <CARRIER_FILE_PATH>
```
- `CARRIER_FILE_PATH`: Path to the image file containing the hidden message.

**Options**
-   `-o`, `--output-path <OUTPUT_FILE_PATH>`: Output path for the decoded message (default: `decoded.txt`).
-   `-k`, `--key <KEY>`: Optional decryption key.

**Example:**
```
mindbender decode hidden.png --output-path revealed_message.txt --key "my_secret_key"
```

## Configuration
MindBender automatically handles lossy images by converting them to a lossless format (PNG). You can provide a custom output path and encryption key as needed.

## License

MindBender is dual-licensed under either:

-   [MIT license](LICENSE-MIT.md)
-   [Apache License, Version 2.0](LICENSE-APACHE.md)

You may choose to use either license.
