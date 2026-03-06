use super::field::{FieldType, FieldTypeError};
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

impl TryFrom<&u8> for FieldDescriptor {
    type Error = FieldDescriptorError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Ok(FieldDescriptor {
            field_type: FieldType::try_from(value)?,
            index: (value >> 3) as usize,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::field::FieldType;

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
}
