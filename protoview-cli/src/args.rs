use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use clap_stdin::MaybeStdin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Can either be passed with the argument or
    /// from STDIN when specifying the value as `-`
    #[arg(short, long, group = "source")]
    pub raw: Option<MaybeStdin<String>>,
    /// Path to a file containing the binary protobuf
    #[arg(short, long, group = "source")]
    pub path: Option<PathBuf>,
    #[arg(short, long, default_value = "auto")]
    pub format: Format,
    #[arg(long, default_value = "false")]
    pub debug: bool
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Hex,
    /// Sequence of 0 and 1s. E.g. `0000100001111011`
    BinaryString,
    /// Binary contentn, e.g. from reading a file directly
    /// Note: It is not possible to auto detect [`Format::Binary`].
    Binary,
    /// In the format of `[u8, u8, u8, ...]`
    U8Array,
    /// In the format of `[i8, i8, i8, ...]`
    I8Array,
    Base64,
    /// Try to detect the format automatically
    Auto,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Hex => write!(f, "hex"),
            Format::BinaryString => write!(f, "binary-string"),
            Format::Binary => write!(f, "binary"),
            Format::U8Array => write!(f, "u8-array"),
            Format::I8Array => write!(f, "i8-array"),
            Format::Base64 => write!(f, "base64"),
            Format::Auto => write!(f, "auto"),
        }
    }
}
