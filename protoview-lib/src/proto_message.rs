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

use crate::{field_type::FieldType, fixed::parse_fixed32, tag::Tag};

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

        match field {
            FieldType::Varint(_) => todo!(),
            FieldType::I64(_) => todo!(),
            FieldType::Len(_) => todo!(),
            FieldType::SGroup(_) => todo!(),
            FieldType::EGroup(_) => todo!(),
            FieldType::I32(_) => {
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 4] = data[idx + 1..idx + 5].try_into().unwrap();
                ret.push(Tag {
                    field: FieldType::I32(parse_fixed32(&bytes) as isize),
                    index: index,
                });
                skip_until = idx + 5;
            }
        }
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

    // #[test]
    // fn test_parse_int() {
    //     assert_eq!(
    //         parse(&[0x08, 0x01]),
    //         vec![Tag {
    //             field: FieldType::Varint(1),
    //             index: 1
    //         }]
    //     );
    //     assert_eq!(
    //         parse(&[0x08, 0x05]),
    //         vec![Tag {
    //             field: FieldType::Varint(5),
    //             index: 1
    //         }]
    //     );
    // }

    // #[test]
    // fn test_parse_multi_int() {
    //     assert_eq!(
    //         parse(&[0x08, 0x01, 0x10, 0x01]),
    //         vec![Tag {
    //             field: FieldType::Varint(1),
    //             index: 1
    //         },Tag {
    //             field: FieldType::Varint(1),
    //             index: 2
    //         }]
    //     );
    //     assert_eq!(
    //         parse(&[0x08, 0x01, 0x18, 0x05]),
    //         vec![Tag {
    //             field: FieldType::Varint(1),
    //             index: 1
    //         },Tag {
    //             field: FieldType::Varint(5),
    //             index: 3
    //         }]
    //     );
    // }

    // #[test]
    // fn test_parse_bigint() {
    //     let binary: [u8; 2] = [0x08, 0x62];

    //     assert_eq!(
    //         parse(&binary),
    //         vec![Tag {
    //             field: FieldType::Varint(5),
    //             index: 0
    //         }]
    //     );
    // }

    // #[test]
    // fn test_parse() {
    //     let binary: [u8; 11] = [
    //         0x08, 0x01, 0x12, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x18, 0x01,
    //     ];

    //     assert_eq!(parse(&binary), vec![]);
    // }
}
