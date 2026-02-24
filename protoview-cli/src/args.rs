use clap::{Parser, ValueEnum};
use clap_stdin::MaybeStdin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Can either be passed with the argument or
    /// from STDIN when specifying the value as `-`
    #[arg(short, long)]
    pub raw: MaybeStdin<String>,
    #[arg(short, long, default_value = "hex")]
    pub format: Format,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Hex,
    Binary,
    /// In the format of [u8, u8, u8]
    DecimalArray,
    Base64,
}
