use clap::Parser;
use thiserror::Error;

use crate::{
    args::Args,
    harmonize_input::{Convert2U8Error, harmonize_input_to_u8},
};
use protoview_lib::parse_proto;

mod args;
mod harmonize_input;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid tag length during: {0}")]
    Harmonize(#[from] Convert2U8Error),
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let input: Vec<u8> = harmonize_input_to_u8(&args.proto_bytes.into_inner(), &args.format)?;
    let parsed = parse_proto(&input);

    println!("{:#?}", parsed);
    Ok(())
}
