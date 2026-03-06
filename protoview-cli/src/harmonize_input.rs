use std::{num::ParseIntError, sync::LazyLock};

use base64::{DecodeError, Engine, prelude::BASE64_STANDARD};
use hex::FromHexError;
use regex::Regex;
use thiserror::Error;

use crate::args::Format;

#[derive(Error, Debug, PartialEq)]
pub enum Convert2U8Error {
    #[error("Invalid hex string passed {0}")]
    DecodeHex(#[from] FromHexError),
    #[error("Invalid base64 string passed {0}")]
    DecodeBase64(#[from] DecodeError),
    #[error("Could not parse as unsigned/signed integer: {0}")]
    Int(#[from] ParseIntError),
    #[error(transparent)]
    DetectFrom(#[from] DetectFormatError),
}

pub(crate) fn harmonize_input_to_u8(
    data: &String,
    format: &Format,
) -> Result<Vec<u8>, Convert2U8Error> {
    match format {
        Format::Hex => Ok(hex::decode(data)?),
        Format::BinaryString => {
            // Split into chunks of 8 bits and convert each to a byte
            Ok(data
                .as_bytes()
                .chunks(8)
                .map(|chunk| {
                    let byte_str = std::str::from_utf8(chunk).unwrap();
                    u8::from_str_radix(byte_str, 2).unwrap()
                })
                .collect())
        }
        Format::Binary => Ok(data.as_bytes().to_vec()),
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
        Format::Base64 => Ok(BASE64_STANDARD.decode(data)?),
        Format::Auto => {
            let detected_format = detect_format(data)?;
            println!("Detected format {detected_format}");
            harmonize_input_to_u8(data, &detected_format)
        }
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
        Ok(Format::BinaryString)
    } else if REGEX_BASE64.is_match(data) {
        Ok(Format::Base64)
    } else {
        Err(DetectFormatError::CouldNotDetect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harmonize_base64() {
        assert_eq!(
            vec![0x08, 0x7b, 0x12, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f],
            harmonize_input_to_u8(&"CHsSBWhlbGxv".to_owned(), &Format::Base64).unwrap()
        )
    }
    #[test]
    fn test_harmonize_hex() {
        assert_eq!(
            vec![0x08, 0x7b, 0x12, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f],
            harmonize_input_to_u8(&"087b120568656c6c6f".to_owned(), &Format::Hex).unwrap()
        )
    }
    #[test]
    fn test_harmonize_binary_string() {
        assert_eq!(
            vec![0x08, 0x7b],
            harmonize_input_to_u8(&"0000100001111011".to_owned(), &Format::BinaryString).unwrap()
        )
    }
    #[test]
    fn test_harmonize_binary() {
        assert_eq!(
            vec![0x08, 0x7b],
            harmonize_input_to_u8(&"\x08\x7b".to_owned(), &Format::Binary).unwrap()
        )
    }
    #[test]
    fn test_harmonize_i8_array() {
        assert_eq!(
            vec![0x08, 0x8b, 0x01],
            harmonize_input_to_u8(&"[8, -117, 1]".to_owned(), &Format::I8Array).unwrap()
        )
    }
    #[test]
    fn test_harmonize_u8_array() {
        assert_eq!(
            vec![0x08, 0x8b, 0x01],
            harmonize_input_to_u8(&"[8, 139, 1]".to_owned(), &Format::U8Array).unwrap()
        )
    }
}
