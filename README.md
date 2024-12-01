
# Mindbender

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

## Introduction

*A steganography CLI tool for encrypting, compressing, hiding, and retrieving text within images.*

### Features

- Text Encoding and Decoding: Hide and retrieve messages within images using LSB steganography
- Encryption: Secure messages with optional AES encryption
- Compression: Optimize hidden data with optional Zlib compression
- Versatile File Handling: Supports lossy image conversion to lossless formats for better encoding

### Building from source

To build Mindbender from source, you will need the following dependencies:

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Once you have installed the dependencies, you can build Mindbender by running the following command:

```bash
cargo build --release
```

This will create a binary file named `target/release/mindbender` in the project directory.

### Installing from source

To install Mindbender from source, you can use the following command:

```bash
cargo install --path .
```

This will install the Mindbender binary in your local Rust installation.

## Usage

### Command-Line Interface (CLI)

Mindbender provides `encode`, `decode`, and `generate-key` commands.

### Generate an encryption key

```
mindbender generate-key [OPTIONS]
```

**Options**
-   `l`, `--length <LENGTH>`: Length of the key to generate in bytes.
-   `o`, `--output <FILE>`: Save the key to a file.

**Example:**
```
mindbender generate-key --length 32 --output key.txt
```

#### Encode a message
```
mindbender encode [OPTIONS] <DATA_FILE_PATH> <CARRIER_FILE_PATH>
```
- `DATA_FILE_PATH`: Path to the text file containing the message to encode.
- `CARRIER_FILE_PATH`: Path to the image file to use as the carrier.

**Options**
-   `-o`, `--output-path <OUTPUT_FILE_PATH>`: Output path for the encoded image (default: `output.png`).
-   `-k`, `--key <KEY>`: Optional encryption key.
-   `-c`, `--compress`: Enable compression (default: `false`).

**Example:**
```
mindbender encode secret_message.txt carrier.jpg --output-path hidden.png --key "my_secret_key" --compress
```

#### Decode a message
```
mindbender decode [OPTIONS] <CARRIER_FILE_PATH>
```
- `CARRIER_FILE_PATH`: Path to the image file containing the hidden message.

**Options**
-   `-o`, `--output-path <OUTPUT_FILE_PATH>`: Output path for the decoded message (default: `decoded.txt`).
-   `-k`, `--key <KEY>`: Optional decryption key.
-   `-d`, `--decompress`: Enable decompression (default: `false`).

**Example:**
```
mindbender decode hidden.png --output-path revealed_message.txt --key "my_secret_key" --decompress
```

### Terminal User Interface (TUI)

Coming soon?

## License

Mindbender is dual-licensed under either:

-   [MIT license](LICENSE-MIT.md)
-   [Apache License, Version 2.0](LICENSE-APACHE.md)

You may choose to use either license.
