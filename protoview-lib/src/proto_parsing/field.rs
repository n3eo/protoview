use std::fmt::{self, Display};
use thiserror::Error;

use crate::{FieldList, i32_to_f32, i64_to_f64};

#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> {
    /// Tag describing the first by of the Tag-Lenght-Value sequence
    pub tag: FieldType,
    pub index: usize,
    pub value: FieldValue<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldType {
    /// int32, int64, uint32, uint64, sint32, sint64, bool, enum
    Varint,
    /// fixed64, sfixed64, double
    I64,
    /// string, bytes, embedded messages, packed repeated fields
    Len,
    /// fixed32, sfixed32, float
    I32,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FieldTypeError {
    #[error("Invalid wire type: {0}")]
    InvalidWireType(usize),
    #[error("Unsupported group wire type: {0}")]
    UnsupportedGroupType(usize),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Varint => write!(f, "Varint"),
            FieldType::I64 => write!(f, "I64"),
            FieldType::Len => write!(f, "Len"),
            FieldType::I32 => write!(f, "I32"),
        }
    }
}

impl TryFrom<&usize> for FieldType {
    type Error = FieldTypeError;

    fn try_from(value: &usize) -> Result<Self, Self::Error> {
        match value & 0b00000111 {
            0b000000 => Ok(FieldType::Varint),
            0b000001 => Ok(FieldType::I64),
            0b000010 => Ok(FieldType::Len),
            0b000011 => Err(FieldTypeError::UnsupportedGroupType(0b000011)),
            0b000100 => Err(FieldTypeError::UnsupportedGroupType(0b000100)),
            0b000101 => Ok(FieldType::I32),
            invalid_type => Err(FieldTypeError::InvalidWireType(invalid_type)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldValue<'a> {
    /// int32, int64, uint32, uint64, sint32, sint64, bool, enum
    Varint(isize),
    /// fixed64, sfixed64, double
    I64(isize),
    /// string, bytes, embedded messages, packed repeated fields
    LenPrimitive(&'a [u8]),
    /// string, bytes, embedded messages, packed repeated fields
    LenSubmessage(FieldList<'a>),
    /// group start/end (deprecated)
    SEGroup(&'a [u8]),
    /// fixed32, sfixed32, float
    I32(isize),
}

impl<'a> fmt::Display for FieldValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::I64(value) => {
                write!(f, "{} | {} | {:e}", value, *value as usize, i64_to_f64(*value as i64))
            }
            FieldValue::I32(value) => {
                write!(f, "{} | {} | {:e}", value, *value as usize, i32_to_f32(*value as i32))
            }
            FieldValue::Varint(value) => {
                write!(f, "{}", value)
            }
            FieldValue::LenPrimitive(items) => write!(f, "{:?}", String::from_utf8(items.to_vec()).unwrap_or_else(|_| format!("{:?}", items))),
            FieldValue::LenSubmessage(fields) => {
                writeln!(f, "[")?; // Blue for submessage start
                for field in fields.0.iter() {
                    writeln!(f, "  {}", field)?;
                }
                write!(f, "]")
            }
            FieldValue::SEGroup(items) => write!(f, "SEGroup({:?})", items),
        }
    }
}

impl Display for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}: {}", self.tag, self.index, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_wire_types() {
        // Test all valid wire types (0-5)
        assert_eq!(FieldType::try_from(&0b000000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b000001).unwrap(), FieldType::I64);
        assert_eq!(FieldType::try_from(&0b000010).unwrap(), FieldType::Len);
        assert_eq!(FieldType::try_from(&0b000101).unwrap(), FieldType::I32);
    }

    #[test]
    fn test_bit_masking() {
        // Test that higher bits are properly masked out
        // These values all have the same lower 4 bits (0b0000) but different higher bits
        assert_eq!(FieldType::try_from(&0b00000000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b00010000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b00100000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b00110000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b01000000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b01010000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b01100000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b01110000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b10000000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b10010000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b10100000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b10110000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b11000000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b11010000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b11100000).unwrap(), FieldType::Varint);
        assert_eq!(FieldType::try_from(&0b11110000).unwrap(), FieldType::Varint);
    }

    #[test]
    fn test_invalid_group_wire_types() {
        let result = FieldType::try_from(&0b0000011);
        assert!(matches!(
            result,
            Err(FieldTypeError::UnsupportedGroupType(3))
        ));
        let result = FieldType::try_from(&0b0000100);
        assert!(matches!(
            result,
            Err(FieldTypeError::UnsupportedGroupType(4))
        ));
    }

    #[test]
    fn test_invalid_wire_type_6() {
        // Test that wire type 6 returns an error
        let result = FieldType::try_from(&0b0000110);
        assert!(matches!(result, Err(FieldTypeError::InvalidWireType(6))));
    }

    #[test]
    fn test_invalid_wire_type_7() {
        // Test that wire type 7 returns an error
        let result = FieldType::try_from(&0b0000111);
        assert!(matches!(result, Err(FieldTypeError::InvalidWireType(7))));
    }

    #[test]
    fn test_invalid_wire_type_15() {
        // Test that wire type 15 returns an error (after masking, it becomes 7)
        let result = FieldType::try_from(&0b0001111);
        assert!(matches!(result, Err(FieldTypeError::InvalidWireType(7))));
    }

    #[test]
    fn test_invalid_wire_type_with_higher_bits() {
        // Test that invalid wire types return an error even with higher bits set
        let result = FieldType::try_from(&0b10001111);
        assert!(matches!(result, Err(FieldTypeError::InvalidWireType(7))));
    }
}
