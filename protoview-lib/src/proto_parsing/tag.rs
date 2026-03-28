use crate::field::{FieldType, FieldTypeError};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct FieldDescriptor {
    pub field_type: FieldType,
    pub index: usize,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FieldDescriptorError {
    #[error("Invalid field descriptor: {0}")]
    InvalidFieldDescriptor(#[from] FieldTypeError),
}

impl TryFrom<&usize> for FieldDescriptor {
    type Error = FieldDescriptorError;

    fn try_from(value: &usize) -> Result<Self, Self::Error> {
        Ok(FieldDescriptor {
            field_type: FieldType::try_from(value)?,
            index: (value >> 3),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{field::FieldType, varint::parse_varint};

    use super::*;

    #[test]
    fn test_get_tag() {
        // Test that get_tag correctly extracts the field number from a tag byte
        // 0x08 = 0b00001000, tag = 0b00001 (1), wire type = 0b000 (0)
        assert_eq!(
            FieldDescriptor::try_from(&0x08).unwrap(),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 1
            }
        );
        // 0x12 = 0b0001001 tag= 0b00010 (2), wire type = 0b010 (2)
        assert_eq!(
            FieldDescriptor::try_from(&0x12).unwrap(),
            FieldDescriptor {
                field_type: FieldType::Len,
                index: 2
            }
        );
        // 0x18 = 0b0001100 tag= 0b00011 (3), wire type = 0b000 (0)
        assert_eq!(
            FieldDescriptor::try_from(&0x18).unwrap(),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 3
            }
        );
    }

    #[test]
    fn test_get_varint_tag() {
        let usize_tag = parse_varint(&[0x80, 0x08]) as usize;
        // 0x80 0X08 = 0b1000000  0b00001000 tag= 0b00011 (3), wire type = 0b000 (0)
        assert_eq!(
            FieldDescriptor::try_from(&usize_tag).unwrap(),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 128
            }
        );

        let usize_tag = parse_varint(&[0x80, 0xf1, 0x04]) as usize;
        assert_eq!(
            FieldDescriptor::try_from(&usize_tag).unwrap(),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 10000
            }
        );
    }
}
