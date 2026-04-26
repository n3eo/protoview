use std::{fs, io};

use clap::Parser;
use thiserror::Error;

use crate::{
    args::Args,
    harmonize_input::{Convert2U8Error, harmonize_input_to_u8},
};
use protoview_lib::{Field, FieldList, ParseProtoError, parse_proto};

use indented_display::{IndentedDisplay, Indenter, Indent};

mod args;
mod harmonize_input;
mod indented_display;

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

    let parsed = parse_proto(&input);

    match parsed {
        Err(e) => eprintln!("{e:?}"),
        Ok(val) => {
            let fields = FieldList(val);
                let indenter = Indenter::new("  ", NoColor {});

                // Create a struct that implements Display using IndentedDisplay
                struct IndentedFieldList<'a> {
                    fields: FieldList<'a>,
                    indenter: Indenter<'a, FieldList<'a>>,
                }

                impl<'a> std::fmt::Display for IndentedFieldList<'a> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        IndentedDisplay::fmt(&self.fields, f, &self.indenter)
                    }
                }

                let indented_fields = IndentedFieldList { fields, indenter };
                println!("{}", indented_fields);
        }
    }
    Ok(())
}
