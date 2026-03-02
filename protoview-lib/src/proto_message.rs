use std::fmt;

use thiserror::Error;

use crate::{
    field::{Field, FieldType, FieldValue},
    fixed::{parse_fixed32, parse_fixed64},
    repeated::find_repeated_length,
    tag::{FieldDescriptor, FieldDescriptorError},
    varint::{find_varint_length, parse_varint},
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseProtoError {
    #[error("Invalid tag length during: {0}")]
    InvalidTagLength(#[from] FieldDescriptorError),
    #[error("Unimplemented protobuf tag")]
    UnimplementedTag,
}

pub fn parse_proto(data: &[u8]) -> Result<Vec<Field<'_>>, ParseProtoError> {
    let mut skip_until: usize = 0;
    let mut ret = vec![];

    for idx in 0..data.len() {
        // Skip bytes that were already processed in a batch
        if idx < skip_until {
            continue;
        }

        // PANIC: for loop guarantees index exists when used without modification
        let byte = data[idx];

        let FieldDescriptor { field_type, index } = FieldDescriptor::try_from(&byte)?;

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

                let len_data = &data[idx + 1 + repeated_length.skip_bytes
                    ..idx + 1 + repeated_length.skip_bytes + repeated_length.length];

                // Check if the first byte of the len data is a valid Protobuf Field Tag as a proxy
                // if the bytes should be parsed as a submessage.
                let test_is_sub_message = FieldDescriptor::try_from(
                    len_data.first().expect("data has a positive length"),
                );

                // As we can't determine the kind of a submessage by looking at the schema
                // we use the field descriptor parsing and message parsing as proxies.
                let field_value = if test_is_sub_message.is_ok() {
                    // If the submessage parsing fails it is a primitiv.
                    if let Ok(sub_message) = parse_proto(len_data) {
                        FieldValue::LenSubmessage(sub_message)
                    } else {
                        FieldValue::LenPrimitive(len_data)
                    }
                } else {
                    FieldValue::LenPrimitive(len_data)
                };
                Field {
                    tag: FieldType::Len,
                    index,
                    value: field_value,
                }
            }
            FieldType::SGroup | FieldType::EGroup => return Err(ParseProtoError::UnimplementedTag),
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

    Ok(ret)
}

/// Wrapper struct for displaying a vector of fields
pub struct FieldList<'a>(pub Vec<Field<'a>>);

impl<'a> fmt::Display for FieldList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in &self.0 {
            write!(f, "{}", field)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed32() {
        let parsed =
            parse_proto(&[0x0d, 0x01, 0x00, 0x00, 0x00, 0x2d, 0x05, 0x00, 0x00, 0x00]).unwrap();
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
        let parsed = parse_proto(&[
            0x09, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ])
        .unwrap();
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
        let parsed = parse_proto(&[0x08, 0x08, 0x10, 0x81, 0x08]).unwrap();
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
        let parsed = parse_proto(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]).unwrap();
        let expected = vec![Field {
            tag: FieldType::Len,
            index: 1,
            value: FieldValue::LenPrimitive(&[0x68, 0x65, 0x6c, 0x6c, 0x6f]),
        }];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_bytes() {
        let parsed = parse_proto(&[0x0a, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f]).unwrap();
        let expected = vec![Field {
            tag: FieldType::Len,
            index: 1,
            value: FieldValue::LenPrimitive(&[104, 101, 108, 108, 111]),
        }];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_repeated_submessage() {
        let parsed = parse_proto(&[0x1a, 0x03, 0x08, 0x96, 0x01]).unwrap();
        let expected = vec![Field {
            tag: FieldType::Len,
            index: 3,
            value: FieldValue::LenSubmessage(vec![Field{
                tag: FieldType::Varint,
                index: 1,
                value: FieldValue::Varint(150)
            }]),
        }];
        assert_eq!(parsed, expected);
    }
}
