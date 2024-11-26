use super::ui::cli::ascii::splash;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const DEFAULT_ENCODED_OUTPUT: &str = "output.png";
const DEFAULT_DECODED_OUTPUT: &str = "decoded.txt";

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Hide and extract sensitive messages in images using steganography",
    long_about = None,
    before_help = splash()
)]
pub struct Cli {
    #[arg(long, help = "Optional name for the operation")]
    pub name: Option<String>,

    #[arg(short, long, value_name = "FILE", help = "Path to configuration file")]
    pub config: Option<PathBuf>,

    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        help = "Enable debug output (use multiple times for more verbosity)"
    )]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

// @todo commands should support piping
#[derive(Subcommand)]
pub enum Commands {
    Encode {
        #[arg(
            value_name = "DATA_FILE_PATH",
            help = "Path to the text file containing the message to encode"
        )]
        data_path: String,

        #[arg(
            value_name = "CARRIER_FILE_PATH",
            help = "Path to the carrier image that will store the message"
        )]
        carrier_path: String,

        #[arg(
            short,
            long,
            value_name = "OUTPUT_FILE_PATH",
            default_value = DEFAULT_ENCODED_OUTPUT,
            help = "Path where the encoded image will be saved"
        )]
        output_path: String,

        #[arg(
            short,
            long,
            value_name = "KEY",
            help = "Optional encryption key to secure the message"
        )]
        key: Option<String>,
    },

    Decode {
        #[arg(
            value_name = "CARRIER_FILE_PATH",
            help = "Path to the image containing the hidden message"
        )]
        carrier_path: String,

        #[arg(
            short,
            long,
            value_name = "OUTPUT_FILE_PATH",
            default_value = DEFAULT_DECODED_OUTPUT,
            help = "Path where the decoded message will be saved"
        )]
        output_path: String,

        #[arg(
            short,
            long,
            value_name = "KEY",
            help = "Decryption key (required if message was encrypted)"
        )]
        key: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_values() {
        let args = ["program", "encode", "message.txt", "input.png"];

        let cli = Cli::parse_from(args);

        match cli.command.unwrap() {
            Commands::Encode { output_path, .. } => {
                assert_eq!(output_path, DEFAULT_ENCODED_OUTPUT);
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_debug_flag_count() {
        let args = ["program", "-ddd", "encode", "message.txt", "input.png"];

        let cli = Cli::parse_from(args);
        assert_eq!(cli.debug, 3);
    }

    #[test]
    fn test_optional_key() {
        let args = [
            "program",
            "encode",
            "message.txt",
            "input.png",
            "--key",
            "secret",
        ];

        let cli = Cli::parse_from(args);

        match cli.command.unwrap() {
            Commands::Encode { key, .. } => {
                assert_eq!(key, Some("secret".to_string()));
            }
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_no_arguments_triggers_tui() {
        let args = ["program"];
        let cli = Cli::parse_from(args);

        assert!(
            cli.command.is_none(),
            "Expected no command, indicating TUI should launch."
        );
    }
}
