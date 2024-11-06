use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "An image-based steganography tool", long_about = None, before_help = ascii_art())]
pub struct Cli {
    pub name: Option<String>,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encode a message into an image
    Encode {
        /// Path to the text file containing the message to encode
        #[arg(value_name = "DATA_FILE_PATH")]
        data_path: String,

        /// Path to the carrier image file
        #[arg(value_name = "CARRIER_FILE_PATH")]
        carrier_path: String,

        /// Output path for the encoded image (default: output.png)
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FILE_PATH",
            default_value = "output.png"
        )]
        output_path: String,

        /// Optional encryption key
        #[arg(short, long, value_name = "KEY")]
        key: Option<String>,
    },

    /// Decode a message from an image
    Decode {
        /// Path to the carrier image file containing the hidden message
        #[arg(value_name = "CARRIER_FILE_PATH")]
        carrier_path: String,

        /// Output path for the decoded message (default: decoded.txt)
        #[arg(
            short,
            long,
            value_name = "OUTPUT_FILE_PATH",
            default_value = "decoded.txt"
        )]
        output_path: String,

        /// Optional decryption key
        #[arg(short, long, value_name = "KEY")]
        key: Option<String>,
    },
}

fn ascii_art() -> &'static str {
    r#"
             ░▒▓██████████████▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓███████▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░
             ░▒▓█▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░

░▒▓███████▓▒░░▒▓████████▓▒░▒▓███████▓▒░░▒▓███████▓▒░░▒▓████████▓▒░▒▓███████▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░
░▒▓███████▓▒░░▒▓██████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓██████▓▒░ ░▒▓███████▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░
░▒▓███████▓▒░░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░
"#
}
