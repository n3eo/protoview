use hex::FromHexError;
use thiserror::Error;

use crate::args::Format;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Convert2U8Error {
    #[error("Invalid hex string passed.")]
    DecodeHexError,
}

pub(crate) fn harmonize_input_to_u8(data: &String, format: &Format) -> Result<Vec<u8>, Convert2U8Error> {
    match format {
        Format::Hex => hex::decode(data).map_err(|_| Convert2U8Error::DecodeHexError),
        Format::Binary => todo!(),
        Format::Decimal => todo!(),
        Format::Base64 => todo!(),
    }
}