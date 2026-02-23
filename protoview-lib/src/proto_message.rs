// struct ProtoMessage {
//     fields: Vec<Box<NodeStruct<dyn Node>>>,
//     data: Vec<u8>,
//     lenght: usize,
// }

// impl ProtoMessage {
//     fn parse(data: Vec<u8>) -> ProtoMessage {
//         let fields = parse(&data);
//         ProtoMessage {
//             fields: fields,
//             data: data,
//             lenght: fields.len(),
//         }
//     }
// }

// fn parse<T>(data: &[u8]) -> Vec<Box<NodeStruct<T>>> {
//     for i in data {}
//     vec![]
// }

use crate::{
    field::{Field, FieldType, FieldValue},
    fixed::{parse_fixed32, parse_fixed64},
    repeated::find_repeated_length,
    tag::FieldDescriptor,
    varint::{find_varint_length, parse_varint},
};

fn parse(data: &[u8]) -> Vec<Field> {
    let mut skip_until: usize = 0;
    let mut ret = vec![];

    for idx in 0..data.len() {
        // Skip bytes that were already processed in a batch
        if idx < skip_until {
            continue;
        }

        // PANIC: for loop guarantees index exists when used without modification
        let byte = data[idx];

        let FieldDescriptor { field_type, index } = FieldDescriptor::from(&byte);

        let tag = match field_type {
            FieldType::Varint => {
                let var_int_len = find_varint_length(&data[idx + 1..]);
                skip_until = idx + 1 + var_int_len;
                Field {
                    tag: FieldType::Varint,
                    index,
                    value: FieldValue::Varint(parse_varint(&data[idx + 1..idx + 1 + var_int_len])),
                }
            }
            FieldType::I64 => {
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 8] = data[idx + 1..idx + 9].try_into().unwrap();
                skip_until = idx + 9;
                Field {
                    tag: FieldType::I64,
                    index,
                    value: FieldValue::I64(parse_fixed64(&bytes) as isize),
                }
            }
            FieldType::Len => {
                let repeated_length = find_repeated_length(&data[idx + 1..]);
                skip_until = idx + 1 + repeated_length.skip_bytes + repeated_length.length;
                Field {
                    tag: FieldType::Len,
                    index,
                    value: FieldValue::LenPrimitive(
                        &data[idx + 1 + repeated_length.skip_bytes
                            ..idx + 1 + repeated_length.skip_bytes + repeated_length.length],
                    ),
                }
            }
            FieldType::SGroup => todo!("Implement deprecated start and end groups"),
            FieldType::EGroup => todo!("Implement deprecated start and end groups"),
            FieldType::I32 => {
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 4] = data[idx + 1..idx + 5].try_into().unwrap();
                skip_until = idx + 5;
                Field {
                    tag: FieldType::I32,
                    index,
                    value: FieldValue::I32(parse_fixed32(&bytes) as isize),
                }
            }
        };
        ret.push(tag);
    }

    ret
}

mod tests {
    use super::*;

    #[test]
    fn test_fixed32() {
        let parsed = parse(&[0x0d, 0x01, 0x00, 0x00, 0x00, 0x2d, 0x05, 0x00, 0x00, 0x00]);
        let expected = vec![
            Field {
                tag: FieldType::I32,
                index: 1,
                value: FieldValue::I32(1),
            },
            Field {
                tag: FieldType::I32,
                index: 5,
                value: FieldValue::I32(5),
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_fixed64() {
        let parsed = parse(&[
            0x09, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        let expected = vec![
            Field {
                tag: FieldType::I64,
                index: 1,
                value: FieldValue::I64(1),
            },
            Field {
                tag: FieldType::I64,
                index: 5,
                value: FieldValue::I64(5),
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_varint() {
        let parsed = parse(&[0x08, 0x08, 0x10, 0x81, 0x08]);
        let expected = vec![
            Field {
                tag: FieldType::Varint,
                index: 1,
                value: FieldValue::Varint(8),
            },
            Field {
                tag: FieldType::Varint,
                index: 2,
                value: FieldValue::Varint(1025),
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_string() {
        let parsed = parse(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        let expected = vec![Field {
            tag: FieldType::Len,
            index: 1,
            value: FieldValue::LenPrimitive(&[0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        }];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_bytes() {
        let parsed = parse(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        let expected = vec![Field {
            tag: FieldType::Len,
            index: 1,
            value: FieldValue::LenPrimitive(&[104, 101, 108, 108, 111]),
        }];
        assert_eq!(parsed, expected);
    }
}
