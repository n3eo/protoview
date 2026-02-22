pub fn parse_fixed32(data: &[u8; 4]) -> i32 {
    let mut ret: isize = 0;
    for i in data.iter().rev() {
        ret = (ret << 8) | (*i as isize);
    }
    ret as i32
}

mod tests {
    use crate::fixed::parse_fixed32;

    #[test]
    fn test_fixed32_parsing() {
        assert_eq!(1600, parse_fixed32(&[0x40, 0x06, 0x00, 0x00]));
        assert_eq!(500, parse_fixed32(&[0xf4, 0x01, 0x00, 0x00]));
        assert_eq!(8192, parse_fixed32(&[0x00, 0x20, 0x00, 0x00]));
    }

    #[test]
    fn test_fixed32_zero() {
        // Test zero value
        assert_eq!(0, parse_fixed32(&[0x00, 0x00, 0x00, 0x00]));
    }

    #[test]
    fn test_fixed32_positive_extremes() {
        // Test maximum positive 32-bit signed integer
        assert_eq!(2147483647, parse_fixed32(&[0xff, 0xff, 0xff, 0x7f]));

        // Test large positive number
        assert_eq!(2005432224, parse_fixed32(&[0xa0, 0x77, 0x88, 0x77]));
    }

    #[test]
    fn test_fixed32_negative_numbers() {
        // As signed
        assert_eq!(-1, parse_fixed32(&[0xff, 0xff, 0xff, 0xff]));
        assert_eq!(-42, parse_fixed32(&[0xd6, 0xff, 0xff, 0xff]));
        assert_eq!(-792, parse_fixed32(&[0xe8, 0xfc, 0xff, 0xff]));

        // As unsigned
        assert_eq!(4294967295, parse_fixed32(&[0xff, 0xff, 0xff, 0xff]) as u32);
        assert_eq!(4294967254, parse_fixed32(&[0xd6, 0xff, 0xff, 0xff]) as u32);
        assert_eq!(4294966504, parse_fixed32(&[0xe8, 0xfc, 0xff, 0xff]) as u32);
    }

    #[test]
    fn test_fixed32_negative_extremes() {
        // Test minimum 32-bit signed integer (-2147483648)
        assert_eq!(-2147483648, parse_fixed32(&[0x00, 0x00, 0x00, 0x80]));

        // Test large negative number
        assert_eq!(-2005432224, parse_fixed32(&[0x60, 0x88, 0x77, 0x88]));
    }

    #[test]
    fn test_fixed32_boundary_values() {
        // Test boundary values around 32-bit limits
        assert_eq!(2147483646, parse_fixed32(&[0xfe, 0xff, 0xff, 0x7f]));
        assert_eq!(-2147483648, parse_fixed32(&[0x00, 0x00, 0x00, 0x80])); // Minimum 32-bit signed integer

        assert_eq!(-2147483647, parse_fixed32(&[0x01, 0x00, 0x00, 0x80]));
        assert_eq!(-2147483646, parse_fixed32(&[0x02, 0x00, 0x00, 0x80]));
    }

    #[test]
    fn test_fixed32_small_values() {
        // Test small positive and negative values
        assert_eq!(1, parse_fixed32(&[0x01, 0x00, 0x00, 0x00]));
        assert_eq!(-1, parse_fixed32(&[0xff, 0xff, 0xff, 0xff]));
        assert_eq!(127, parse_fixed32(&[0x7f, 0x00, 0x00, 0x00]));
        assert_eq!(-128, parse_fixed32(&[0x80, 0xff, 0xff, 0xff]));
    }
}
