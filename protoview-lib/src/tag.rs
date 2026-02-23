use crate::field::{FieldValue, FieldType};

#[derive(Debug, PartialEq, Eq)]
pub struct FieldDescriptor {
    pub field_type: FieldType,
    pub index: usize,
}

impl From<&u8> for FieldDescriptor {
    fn from(value: &u8) -> Self {
        FieldDescriptor {
            field_type: FieldType::from(value),

            index: (value >> 3) as usize,
        }
    }
}

mod tests {
    use crate::field::FieldType;

    use super::*;

    #[test]
    fn test_get_tag() {
        // Test that get_tag correctly extracts the field number from a tag byte
        // 0x08 = 0b00001000, tag = 0b00001 (1), wire type = 0b000 (0)
        assert_eq!(
            FieldDescriptor::from(&0x08),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 1
            }
        );
        // 0x12 = 0b0001001 tag= 0b00010 (2), wire type = 0b010 (2)
        assert_eq!(
            FieldDescriptor::from(&0x12),
            FieldDescriptor {
                field_type: FieldType::Len,
                index: 2
            }
        );
        // 0x18 = 0b0001100 tag= 0b00011 (3), wire type = 0b000 (0)
        assert_eq!(
            FieldDescriptor::from(&0x18),
            FieldDescriptor {
                field_type: FieldType::Varint,
                index: 3
            }
        );
    }
}
