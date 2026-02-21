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

use crate::{field_type::FieldType, tag::Tag};

fn parse(data: &[u8]) -> Vec<Tag> {
    let mut idx: usize = 0;
    let mut ret = vec![];

    let mut next_type = None;
    for i in data {
        match next_type {
            Some(FieldType::Varint(v)) => {
                if *i & 0b1000000 == 0b1000000 {
                    todo!("Implement integer continue");
                    continue;
                } else {
                    ret.push(Tag {
                        field: FieldType::Varint(v + *i as i8 as isize),
                        index: idx,
                    });
                    next_type = None;
                    continue;
                }
            }
            None => (),
            _ => (),
        }

        let Tag {
            field: field_type,
            index,
        } = Tag::from(i);
        next_type = Some(field_type);
        idx = index;
    }
    ret
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        assert_eq!(
            parse(&[0x08, 0x01]),
            vec![Tag {
                field: FieldType::Varint(1),
                index: 1
            }]
        );
        assert_eq!(
            parse(&[0x08, 0x05]),
            vec![Tag {
                field: FieldType::Varint(5),
                index: 1
            }]
        );
    }

    #[test]
    fn test_parse_multi_int() {
        assert_eq!(
            parse(&[0x08, 0x01, 0x10, 0x01]),
            vec![Tag {
                field: FieldType::Varint(1),
                index: 1
            },Tag {
                field: FieldType::Varint(1),
                index: 2
            }]
        );
        assert_eq!(
            parse(&[0x08, 0x01, 0x18, 0x05]),
            vec![Tag {
                field: FieldType::Varint(1),
                index: 1
            },Tag {
                field: FieldType::Varint(5),
                index: 3
            }]
        );
    }

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
