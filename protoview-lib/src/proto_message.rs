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
    field_type::FieldType,
    fixed::{parse_fixed32, parse_fixed64},
    repeated::find_repeated_length,
    tag::Tag,
    varint::{find_varint_length, parse_varint},
};

fn parse(data: &[u8]) -> Vec<Tag> {
    let mut skip_until: usize = 0;
    let mut ret = vec![];

    for idx in 0..data.len() {
        // Skip bytes that were already processed in a batch
        if idx < skip_until {
            continue;
        }

        // PANIC: for loop guarantees index exists when used without modification
        let byte = data[idx];

        let Tag { field, index } = Tag::from(&byte);

        let tag = match field {
            FieldType::Varint(_) => {
                let var_int_len = find_varint_length(&data[idx + 1..]);
                skip_until = idx + 1 + var_int_len;
                Tag {
                    field: FieldType::Varint(parse_varint(&data[idx + 1..idx + 1 + var_int_len])),
                    index: index,
                }
            }
            FieldType::I64(_) => {
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 8] = data[idx + 1..idx + 9].try_into().unwrap();
                skip_until = idx + 9;
                Tag {
                    field: FieldType::I64(parse_fixed64(&bytes) as isize),
                    index: index,
                }
            }
            FieldType::Len(_) => {
                let repeated_length = find_repeated_length(&data[idx + 1..]);
                skip_until = idx + 1 + repeated_length.skip_bytes + repeated_length.length;
                Tag {
                    field: FieldType::Len(
                        &data[idx + 1 + repeated_length.skip_bytes
                            ..idx + 1 + repeated_length.skip_bytes + repeated_length.length],
                    ),
                    index,
                }
            }
            FieldType::SGroup => todo!("Implement deprecated start and end groups"),
            FieldType::EGroup => todo!("Implement deprecated start and end groups"),
            FieldType::I32(_) => {
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 4] = data[idx + 1..idx + 5].try_into().unwrap();
                skip_until = idx + 5;
                Tag {
                    field: FieldType::I32(parse_fixed32(&bytes) as isize),
                    index: index,
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
            Tag {
                field: FieldType::I32(1),
                index: 1,
            },
            Tag {
                field: FieldType::I32(5),
                index: 5,
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
            Tag {
                field: FieldType::I64(1),
                index: 1,
            },
            Tag {
                field: FieldType::I64(5),
                index: 5,
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_varint() {
        let parsed = parse(&[0x08, 0x08, 0x10, 0x81, 0x08]);
        let expected = vec![
            Tag {
                field: FieldType::Varint(8),
                index: 1,
            },
            Tag {
                field: FieldType::Varint(1025),
                index: 2,
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_string() {
        let parsed = parse(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        let expected = vec![
            Tag {
                field: FieldType::Len(&[0x68, 0x65, 0x6c, 0x6c, 0x6f]),
                index: 1,
            },
        ];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_bytes() {
        let parsed = parse(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        let expected = vec![
            Tag {
                field: FieldType::Len(&[104, 101, 108, 108, 111]),
                index: 1,
            },
        ];
        assert_eq!(parsed, expected);
    }
}
