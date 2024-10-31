use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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
    Encode {
        #[arg(value_name = "DATA_FILE_PATH")]
        data_path: String,

        #[arg(value_name = "CARRIER_FILE_PATH")]
        carrier_path: String,

        #[arg(short, long, value_name = "OUTPUT_FILE_PATH", default_value = "output")]
        output_path: String,

        #[arg(short, long, value_name = "KEY")]
        key: Option<String>,
    },

    Decode {
        #[arg(value_name = "CARRIER_FILE_PATH")]
        carrier_path: String,

        #[arg(
            short,
            long,
            value_name = "OUTPUT_FILE_PATH",
            default_value = "decoded"
        )]
        output_path: String,

        #[arg(short, long, value_name = "KEY")]
        key: Option<String>,
    },
}
