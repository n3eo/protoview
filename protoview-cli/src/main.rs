use std::{fs, io};

use clap::Parser;
use thiserror::Error;

use crate::{
    args::Args,
    harmonize_input::{Convert2U8Error, harmonize_input_to_u8},
};
use protoview_lib::{FieldList, parse_proto};

mod args;
mod harmonize_input;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid tag length during: {0}")]
    Harmonize(#[from] Convert2U8Error),
    #[error("Could not read the provided path: {0}")]
    ReadFile(#[from] io::Error),
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

    let parsed = parse_proto(&input);

    match parsed {
        Err(e) => eprintln!("{e:?}"),
        Ok(val) => {
            if args.debug {
                println!("{:#?}", val);
            } else {
                println!("{}", FieldList(val));
            }
        }
    }
    Ok(())
}
