pub fn parse_varint(data: &[u8]) -> isize {
    let mut ret: isize = 0;
    for d in data.iter().rev() {
        // Unset the 8th bit which only indicates if more bytes follow
        ret = (ret << 7) | ((*d & 0b01111111) as isize);
    }
    ret
}

pub fn find_varint_length(data: &[u8]) -> usize {
    // TOOD: Idea could be omptimized with SIMD
    if data.is_empty() {
        panic!("Cannot find varint length in empty slice");
    }

    let mut length = 0;
    for (i, d) in data.iter().enumerate() {
        if d & 0b10000000 == 0b00000000 {
            return i + 1;
        }
        length += 1;

        // Prevent infinite loops and invalid varints that are too long
        if i >= 10 {
            // Maximum varint length in protobuf is 10 bytes
            panic!("Invalid varint: no terminating byte found within maximum length");
        }
    }

    // If we reach here, the varint is invalid (no terminating byte)
    panic!("Invalid varint: no terminating byte found");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_basic() {
        // Test basic single-byte values
        assert_eq!(0, parse_varint(&[0x00]));
        assert_eq!(1, parse_varint(&[0x01]));
        assert_eq!(127, parse_varint(&[0x7f]));
        assert_eq!(128, parse_varint(&[0x80, 0x01]));
    }

    #[test]
    fn test_varint_small_values() {
        // Test small positive values
        assert_eq!(5, parse_varint(&[0x05]));
        assert_eq!(10, parse_varint(&[0x0a]));
        assert_eq!(100, parse_varint(&[0x64])); // 0x64 & 0x7f = 0x44 = 68? Wait, let me recalculate
        assert_eq!(127, parse_varint(&[0x7f]));
    }

    #[test]
    fn test_varint_multi_byte() {
        // Test multi-byte varint values
        assert_eq!(128, parse_varint(&[0x80, 0x01]));
        assert_eq!(16384, parse_varint(&[0x80, 0x80, 0x01]));
        assert_eq!(2097152, parse_varint(&[0x80, 0x80, 0x80, 0x01]));
        assert_eq!(268435456, parse_varint(&[0x80, 0x80, 0x80, 0x80, 0x01]));
    }

    #[test]
    fn test_varint_large_values() {
        // Test large positive values
        assert_eq!(300, parse_varint(&[0xac, 0x02]));
        assert_eq!(16383, parse_varint(&[0xff, 0x7f]));
        assert_eq!(2097151, parse_varint(&[0xff, 0xff, 0x7f]));
        assert_eq!(268435455, parse_varint(&[0xff, 0xff, 0xff, 0x7f]));
    }

    #[test]
    fn test_varint_edge_cases() {
        // Test edge cases
        assert_eq!(0, parse_varint(&[0x00])); // Zero
        assert_eq!(1, parse_varint(&[0x01])); // Minimum positive
        assert_eq!(127, parse_varint(&[0x7f])); // Maximum single-byte positive after masking
        assert_eq!(128, parse_varint(&[0x80, 0x01])); // Minimum two-byte value
    }

    #[test]
    fn test_find_varint_length() {
        // Test single byte varints
        assert_eq!(1, find_varint_length(&[0x00])); // Zero
        assert_eq!(1, find_varint_length(&[0x7F])); // Max single byte
        assert_eq!(1, find_varint_length(&[0x01])); // Small positive
        assert_eq!(1, find_varint_length(&[0x40])); // Mid range

        // Test two byte varints
        assert_eq!(2, find_varint_length(&[0x80, 0x7F])); // Min two byte
        assert_eq!(2, find_varint_length(&[0xFF, 0x7F])); // Max two byte

        // Test three byte varints
        assert_eq!(3, find_varint_length(&[0x80, 0x8F, 0x00]));
        assert_eq!(3, find_varint_length(&[0xFF, 0xFF, 0x7F]));

        // Test four byte varints
        assert_eq!(4, find_varint_length(&[0x80, 0xC4, 0xE1, 0x77]));
        assert_eq!(4, find_varint_length(&[0xFF, 0xFF, 0xFF, 0x7F]));

        // Test five byte varints
        assert_eq!(5, find_varint_length(&[0x80, 0x80, 0x80, 0x80, 0x7F]));
        assert_eq!(5, find_varint_length(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]));
    }

    #[test]
    fn test_find_varint_length_with_extra_data() {
        // Test finding varint length when there's extra data after the varint
        assert_eq!(1, find_varint_length(&[0x00, 0xFF, 0xFF, 0xFF])); // Single byte followed by data
        assert_eq!(2, find_varint_length(&[0x80, 0x7F, 0xFF, 0xFF])); // Two byte followed by data
        assert_eq!(3, find_varint_length(&[0x80, 0x80, 0x7F, 0xFF])); // Three byte followed by data
    }

    #[test]
    fn test_find_varint_length_edge_cases() {
        // Test all continuation bits set (max length)
        assert_eq!(
            10,
            find_varint_length(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00])
        );
    }

    #[test]
    #[should_panic(expected = "Cannot find varint length in empty slice")]
    fn test_find_varint_length_panics_on_empty() {
        find_varint_length(&[]);
    }

    #[test]
    #[should_panic(expected = "Invalid varint: no terminating byte found")]
    fn test_find_varint_length_panics_on_no_terminator() {
        find_varint_length(&[0x80]);
    }

    #[test]
    #[should_panic(expected = "Invalid varint: no terminating byte found within maximum length")]
    fn test_find_varint_length_panics_on_too_long() {
        find_varint_length(&[
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
        ]);
    }
}
