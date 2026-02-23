use crate::varint::{find_varint_length, parse_varint};

#[derive(Debug, PartialEq, Eq)]
pub struct RepeatedLength {
    /// length of the repeated field
    pub length: usize,
    /// Defines the number of bytes the length was encoded in
    /// to know after how many bytes of var int encoded length
    /// the actual values start.
    pub skip_bytes: usize,
}

/// Extract the lenght of a repeated field from the passed bytes.
/// The bytes start with the length of the repeated field directly.
pub fn find_repeated_length(data: &[u8]) -> RepeatedLength {
    let var_int_len = find_varint_length(data);
    let length = parse_varint(&data[..var_int_len]) as usize;

    RepeatedLength {
        length,
        skip_bytes: var_int_len,
    }
}

mod tests {
    use super::*;

    #[test]
    pub fn test_find_repeated_length() {
        assert_eq!(
            RepeatedLength {
                length: 5,
                skip_bytes: 1
            },
            find_repeated_length(&[0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f])
        )
    }

    #[test]
    fn test_find_repeated_length_single_byte() {
        // Test various single-byte varint lengths
        assert_eq!(
            RepeatedLength {
                length: 0,
                skip_bytes: 1
            },
            find_repeated_length(&[0x00, 0xFF, 0xFF])
        );
        assert_eq!(
            RepeatedLength {
                length: 1,
                skip_bytes: 1
            },
            find_repeated_length(&[0x01, 0xAA, 0xBB])
        );
        assert_eq!(
            RepeatedLength {
                length: 127,
                skip_bytes: 1
            },
            find_repeated_length(&[0x7F, 0xCC, 0xDD])
        );
    }

    #[test]
    fn test_find_repeated_length_multi_byte() {
        // Test multi-byte varint lengths
        assert_eq!(
            RepeatedLength {
                length: 128,
                skip_bytes: 2
            },
            find_repeated_length(&[0x80, 0x01, 0xEE, 0xFF])
        );
        assert_eq!(
            RepeatedLength {
                length: 16384,
                skip_bytes: 3
            },
            find_repeated_length(&[0x80, 0x80, 0x01, 0x11, 0x22])
        );
        assert_eq!(
            RepeatedLength {
                length: 2097152,
                skip_bytes: 4
            },
            find_repeated_length(&[0x80, 0x80, 0x80, 0x01, 0x33, 0x44])
        );
    }

    #[test]
    fn test_find_repeated_length_edge_cases() {
        // Test edge cases
        assert_eq!(
            RepeatedLength {
                length: 0,
                skip_bytes: 1
            },
            find_repeated_length(&[0x00]) // Zero length with no data
        );
        assert_eq!(
            RepeatedLength {
                length: 300,
                skip_bytes: 2
            },
            find_repeated_length(&[0xAC, 0x02, 0x55, 0x66])
        );
        assert_eq!(
            RepeatedLength {
                length: 16383,
                skip_bytes: 2
            },
            find_repeated_length(&[0xFF, 0x7F, 0x77, 0x88])
        );
    }

    #[test]
    fn test_find_repeated_length_large_values() {
        // Test large values that require multi-byte encoding
        assert_eq!(
            RepeatedLength {
                length: 262143,
                skip_bytes: 3
            },
            find_repeated_length(&[0xFF, 0xFF, 0x0F, 0x99, 0xAA])
        );
        assert_eq!(
            RepeatedLength {
                length: 33554431,
                skip_bytes: 4
            },
            find_repeated_length(&[0xFF, 0xFF, 0xFF, 0x0F, 0xBB, 0xCC])
        );
    }

    #[test]
    #[should_panic(expected = "Cannot find varint length in empty slice")]
    fn test_find_repeated_length_panics_on_empty() {
        find_repeated_length(&[]);
    }

    #[test]
    #[should_panic(expected = "Invalid varint: no terminating byte found")]
    fn test_find_repeated_length_panics_on_invalid_varint() {
        // Test with invalid varint (no termination byte)
        find_repeated_length(&[0x80]);
    }
}
