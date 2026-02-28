use std::{num::ParseIntError, sync::LazyLock};

use hex::FromHexError;
use regex::Regex;
use thiserror::Error;

use crate::args::Format;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Convert2U8Error {
    #[error("Invalid hex string passed.")]
    DecodeHex,
    #[error("Could not parse as unsigned integer: {0}")]
    UInt(#[from] ParseIntError),
    #[error(transparent)]
    DetectFrom(#[from] DetectFormatError),
}

pub(crate) fn harmonize_input_to_u8(
    data: &String,
    format: &Format,
) -> Result<Vec<u8>, Convert2U8Error> {
    match format {
        Format::Hex => hex::decode(data).map_err(|_| Convert2U8Error::DecodeHex),
        Format::Binary => todo!(),
        Format::U8Array => data
            .trim_start_matches("[")
            .trim_end_matches("]")
            .split(",")
            .map(|s| Ok(s.trim().parse::<u8>()?))
            .collect(),
        Format::I8Array => data
            .trim_start_matches("[")
            .trim_end_matches("]")
            .split(",")
            .map(|s| Ok(s.trim().parse::<i8>()? as u8))
            .collect(),
        Format::Base64 => todo!(),
        Format::Auto => {
            let detected_format = detect_format(data)?;
            println!("Detected format {detected_format}");
            harmonize_input_to_u8(data, &detected_format)}
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DetectFormatError {
    #[error("Could not detect the format of th input data")]
    CouldNotDetect,
}

static REGEX_U8: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^\[([0-9]{1,3}[[:space:]]?,[[:space:]]?)+[0-9]{1,3}\]$")
        .expect("Static regex does not cange and is valid.")
});
static REGEX_I8: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^\[(-?[0-9]{1,3}[[:space:]]?,[[:space:]]?)+-?[0-9]{1,3}\]$")
        .expect("Static regex does not cange and is valid.")
});
static REGEX_HEX: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[a-fA-F0-9]+$").expect("Static regex does not cange and is valid.")
});
static REGEX_BINARY: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^[01]+$").expect("Static regex does not cange and is valid.")
});
static REGEX_BASE64: LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$")
        .expect("Static regex does not cange and is valid.")
});

fn detect_format(data: &str) -> Result<Format, DetectFormatError> {
    if REGEX_U8.is_match(data) {
        Ok(Format::U8Array)
    } else if REGEX_I8.is_match(data) {
        Ok(Format::I8Array)
    } else if REGEX_HEX.is_match(data) {
        Ok(Format::Hex)
    } else if REGEX_BINARY.is_match(data) {
        Ok(Format::Binary)
    } else if REGEX_BASE64.is_match(data) {
        Ok(Format::Base64)
    } else {
        Err(DetectFormatError::CouldNotDetect)
    }
}
