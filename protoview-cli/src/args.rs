use clap::{Parser, ValueEnum};
use clap_stdin::MaybeStdin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub proto_bytes: MaybeStdin<String>,
    #[arg(short, long, default_value = "hex")]
    pub format: Format,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Hex,
    Binary,
    Decimal,
    Base64,
}
