use std::fmt;
use thiserror::Error;

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
    /// group start (deprecated)
    SGroup,
    /// group end (deprecated)
    EGroup,
    /// fixed32, sfixed32, float
    I32,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FieldTypeError {
    #[error("Invalid wire type: {0}")]
    InvalidWireType(u8),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Varint => write!(f, "Varint"),
            FieldType::I64 => write!(f, "I64"),
            FieldType::Len => write!(f, "Len"),
            FieldType::SGroup => write!(f, "SGroup"),
            FieldType::EGroup => write!(f, "EGroup"),
            FieldType::I32 => write!(f, "I32"),
        }
    }
}

impl TryFrom<&u8> for FieldType {
    type Error = FieldTypeError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value & 0b00000111 {
            0b000000 => Ok(FieldType::Varint),
            0b000001 => Ok(FieldType::I64),
            0b000010 => Ok(FieldType::Len),
            0b000011 => Ok(FieldType::SGroup),
            0b000100 => Ok(FieldType::EGroup),
            0b000101 => Ok(FieldType::I32),
            invalid_type => Err(FieldTypeError::InvalidWireType(invalid_type)),
        }
    }
}

impl<'a> fmt::Display for FieldValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Varint(value) => write!(f, "{}", value),
            FieldValue::I64(value) => write!(f, "{}", value),
            FieldValue::LenPrimitive(bytes) => {
                // Try to display as UTF-8 string if possible, otherwise show as hex
                if let Ok(string) = std::str::from_utf8(bytes) {
                    write!(f, "\"{}\"", string)
                } else {
                    write!(f, "{:?}", bytes)
                }
            }
            FieldValue::LenSubmessage(fields) => {
                writeln!(f, "[")?;
                for field in fields.iter() {
                    write!(f, "  {}", field)?;
                }
                write!(f, "]")
            }
            FieldValue::SGroup => write!(f, "SGroup"),
            FieldValue::EGroup => write!(f, "EGroup"),
            FieldValue::I32(value) => write!(f, "{}", value),
        }
    }
}

impl<'a> Field<'a> {
    /// Display the field with optional indentation
    fn display_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        write!(f, "{}", self.index)?;
        match &self.value {
            FieldValue::LenSubmessage(fields) => {
                writeln!(f, ": SubMessage = {{")?;
                let new_indent = format!("{}{}", indent, "    ");
                for field in fields.iter() {
                    write!(f, "{}    ", indent)?;
                    field.display_with_indent(f, &new_indent)?;
                }
                writeln!(f, "{}}}", indent)?;
            }
            _ => writeln!(f, ": {} = {}", self.tag, self.value)?,
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_with_indent(f, "")
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
    LenSubmessage(Vec<Field<'a>>),
    /// group start (deprecated)
    SGroup,
    /// group end (deprecated)
    EGroup,
    /// fixed32, sfixed32, float
    I32(isize),
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
        assert_eq!(FieldType::try_from(&0b000011).unwrap(), FieldType::SGroup);
        assert_eq!(FieldType::try_from(&0b000100).unwrap(), FieldType::EGroup);
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
    fn test_enum_equality() {
        // Test that FieldType variants can be compared for equality
        assert_eq!(FieldType::Varint, FieldType::Varint);
        assert_eq!(FieldType::I64, FieldType::I64);
        assert_eq!(FieldType::Len, FieldType::Len);
        assert_eq!(FieldType::SGroup, FieldType::SGroup);
        assert_eq!(FieldType::EGroup, FieldType::EGroup);
        assert_eq!(FieldType::I32, FieldType::I32);

        // Test inequality
        assert_ne!(FieldType::Varint, FieldType::I64);
        assert_ne!(FieldType::Varint, FieldType::Len);
        assert_ne!(FieldType::Varint, FieldType::SGroup);
        assert_ne!(FieldType::Varint, FieldType::EGroup);
        assert_ne!(FieldType::Varint, FieldType::I32);
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

    #[test]
    fn test_display_implementation() {
        // Test FieldType display
        assert_eq!(format!("{}", FieldType::Varint), "Varint");
        assert_eq!(format!("{}", FieldType::I64), "I64");
        assert_eq!(format!("{}", FieldType::Len), "Len");
        assert_eq!(format!("{}", FieldType::I32), "I32");

        // Test FieldValue display
        assert_eq!(format!("{}", FieldValue::Varint(42)), "42");
        assert_eq!(format!("{}", FieldValue::I32(123)), "123");
        assert_eq!(
            format!("{}", FieldValue::LenPrimitive(b"hello")),
            "\"hello\""
        );
        assert_eq!(
            format!("{}", FieldValue::LenPrimitive(&[0xFF, 0xFE])),
            "[255, 254]"
        );

        // Test Field display
        let field = Field {
            tag: FieldType::Varint,
            index: 1,
            value: FieldValue::Varint(42),
        };
        assert_eq!(format!("{}", field), "1: Varint = 42\n");

        let string_field = Field {
            tag: FieldType::Len,
            index: 2,
            value: FieldValue::LenPrimitive(b"test string"),
        };
        assert_eq!(format!("{}", string_field), "2: Len = \"test string\"\n");
    }
}
