#[derive(Debug, PartialEq, Eq)]
pub enum FieldType<'a> {
    /// int32, int64, uint32, uint64, sint32, sint64, bool, enum
    Varint(isize),
    /// fixed64, sfixed64, double
    I64(isize),
    /// string, bytes, embedded messages, packed repeated fields
    Len(&'a [u8]),
    /// group start (deprecated)
    SGroup,
    /// group end (deprecated)
    EGroup,
    /// fixed32, sfixed32, float
    I32(isize),
}

impl<'a> From<&u8> for FieldType<'a> {
    fn from(value: &u8) -> Self {
        match value & 0b00000111 {
            0b000000 => FieldType::Varint(0),
            0b000001 => FieldType::I64(0),
            0b000010 => FieldType::Len(&[]),
            0b000011 => FieldType::SGroup,
            0b000100 => FieldType::EGroup,
            0b000101 => FieldType::I32(0),
            _ => panic!("Incorrect wire type"), // TODO: Refactor to TryFrom and don't panic!
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_wire_types() {
        // Test all valid wire types (0-5)
        assert_eq!(FieldType::from(&0b000000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b000001), FieldType::I64(0));
        assert_eq!(FieldType::from(&0b000010), FieldType::Len(&[]));
        assert_eq!(FieldType::from(&0b000011), FieldType::SGroup);
        assert_eq!(FieldType::from(&0b000100), FieldType::EGroup);
        assert_eq!(FieldType::from(&0b000101), FieldType::I32(0));
    }

    #[test]
    fn test_bit_masking() {
        // Test that higher bits are properly masked out
        // These values all have the same lower 4 bits (0b0000) but different higher bits
        assert_eq!(FieldType::from(&0b00000000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b00010000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b00100000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b00110000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b01000000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b01010000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b01100000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b01110000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b10000000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b10010000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b10100000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b10110000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b11000000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b11010000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b11100000), FieldType::Varint(0));
        assert_eq!(FieldType::from(&0b11110000), FieldType::Varint(0));
    }

    #[test]
    fn test_enum_equality() {
        // Test that FieldType variants can be compared for equality
        assert_eq!(FieldType::Varint(0), FieldType::Varint(0));
        assert_eq!(FieldType::I64(0), FieldType::I64(0));
        assert_eq!(FieldType::Len(&[]), FieldType::Len(&[]));
        assert_eq!(FieldType::SGroup, FieldType::SGroup);
        assert_eq!(FieldType::EGroup, FieldType::EGroup);
        assert_eq!(FieldType::I32(0), FieldType::I32(0));

        // Test inequality
        assert_ne!(FieldType::Varint(0), FieldType::I64(0));
        assert_ne!(FieldType::Varint(0), FieldType::Len(&[]));
        assert_ne!(FieldType::Varint(0), FieldType::SGroup);
        assert_ne!(FieldType::Varint(0), FieldType::EGroup);
        assert_ne!(FieldType::Varint(0), FieldType::I32(0));
    }

    #[test]
    #[should_panic(expected = "Incorrect wire type")]
    fn test_invalid_wire_type_6() {
        // Test that wire type 6 panics
        let _ = FieldType::from(&0b0000110);
    }

    #[test]
    #[should_panic(expected = "Incorrect wire type")]
    fn test_invalid_wire_type_7() {
        // Test that wire type 7 panics
        let _ = FieldType::from(&0b0000111);
    }

    #[test]
    #[should_panic(expected = "Incorrect wire type")]
    fn test_invalid_wire_type_15() {
        // Test that wire type 15 panics
        let _ = FieldType::from(&0b0001111);
    }

    #[test]
    #[should_panic(expected = "Incorrect wire type")]
    fn test_invalid_wire_type_with_higher_bits() {
        // Test that invalid wire types panic even with higher bits set
        let _ = FieldType::from(&0b10001111);
    }
}
