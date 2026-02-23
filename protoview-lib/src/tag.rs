use crate::field_type::FieldType;

#[derive(Debug, PartialEq, Eq)]
pub struct Tag<'a> {
    pub field: FieldType<'a>,
    pub index: usize,
}

impl<'a> From<&u8> for Tag<'a> {
    fn from(value: &u8) -> Self {
        Tag {
            field: FieldType::from(value),
            index: (value >> 3) as usize,
        }
    }
}

mod tests {
    use crate::field_type::FieldType;

    use super::*;

    #[test]
    fn test_get_tag() {
        // Test that get_tag correctly extracts the field number from a tag byte
        // 0x08 = 0b00001000, tag = 0b00001 (1), wire type = 0b000 (0)
        assert_eq!(
            Tag::from(&0x08),
            Tag {
                field: FieldType::Varint(0),
                index: 1
            }
        );
        // 0x12 = 0b0001001 tag= 0b00010 (2), wire type = 0b010 (2)
        assert_eq!(
            Tag::from(&0x12),
            Tag {
                field: FieldType::Len(&[]),
                index: 2
            }
        );
        // 0x18 = 0b0001100 tag= 0b00011 (3), wire type = 0b000 (0)
        assert_eq!(
            Tag::from(&0x18),
            Tag {
                field: FieldType::Varint(0),
                index: 3
            }
        );
    }
}
