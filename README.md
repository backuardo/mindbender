
# Mindbender

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

*A steganography tool for encrypting and hiding text within images.*

## Introduction

**Mindbender** is a CLI tool for hiding (and retrieving) text data within images using steganography techniques.

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

Mindbender provides `encode` and `decode` commands.

#### Encode a message
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

#### Decode a message
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

### Terminal User Interface (TUI)

Coming soon?

## License

Mindbender is dual-licensed under either:

-   [MIT license](LICENSE-MIT.md)
-   [Apache License, Version 2.0](LICENSE-APACHE.md)

You may choose to use either license.
