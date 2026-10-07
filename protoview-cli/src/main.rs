use clap::Parser;
use protoview_lib::{FieldList, ParseProtoError, parse_proto};
use std::{fs, io, time::Instant};
use thiserror::Error;

use crate::{args::Args, display::Styled, harmonize_input::{Convert2U8Error, harmonize_input_to_u8}};

mod args;
mod display;
mod harmonize_input;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid tag length during: {0}")]
    Harmonize(#[from] Convert2U8Error),
    #[error("Could not read the provided path: {0}")]
    ReadFile(#[from] io::Error),
    #[error("Error parsing the protobuf message: {0}")]
    ParseProto(#[from] ParseProtoError),
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let input = match args.path {
        Some(path) => fs::read(path)?,
        None => args
            .raw
            .map(|raw| harmonize_input_to_u8(&raw.into_inner(), &args.format))
            .expect("Neither a file or a raw input is defined")?,
    };

    let start = Instant::now();
    let parsed = parse_proto(&input);
    let duration = start.elapsed();

    if args.debug {
        println!("{:#?}", parsed?);
    } else {
        // Colors and indentation are independent settings of the same
        // printing routine: `--color` toggles the scheme, indentation is
        // the padding passed to `indented`.
        let fields = FieldList(parsed?);
        let styled = Styled::plain(&fields).indented("  ");
        if args.color {
            println!("{}", styled.colored());
        } else {
            println!("{}", styled);
        }
    }
    println!("Parsing took: {:?}", duration);
    Ok(())
}
