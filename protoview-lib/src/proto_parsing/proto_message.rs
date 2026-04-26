use super::{
    field::{Field, FieldType, FieldTypeError, FieldValue},
    fixed::{parse_fixed32, parse_fixed64},
    repeated::find_repeated_length,
    tag::{FieldDescriptor, FieldDescriptorError},
    varint::{find_varint_length, parse_varint},
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseProtoError<'a> {
    #[error("Invalid tag length during: {0}")]
    InvalidTagLength(#[from] FieldDescriptorError),
    #[error("Data is considered incomplete after parsing {0:?} fields with {1} remaining bytes.")]
    IncompleteData(Vec<Field<'a>>, usize),
    #[error("Malformed protobuf (sub)message of {0:?}")]
    MalformedProtobuf(Vec<u8>),
}

pub fn parse_proto(data: &'_ [u8]) -> Result<Vec<Field<'_>>, ParseProtoError<'_>> {
    let mut skip_until: usize = 0;
    let mut ret = vec![];

    for idx in 0..data.len() {
        // Skip bytes that were already processed in a batch
        if idx < skip_until {
            continue;
        }

        // Check if we have enough data for tag varint
        let tag_varint_len = find_varint_length(&data[idx..]);
        if idx + tag_varint_len > data.len() {
            return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
        }

        // Protobuf tag field can't be a signed integer
        let tag_bytes = parse_varint(&data[idx..idx + tag_varint_len]) as usize;

        let field_descriptor_result = FieldDescriptor::try_from(&tag_bytes);

        let (field_type, index) = match field_descriptor_result {
            Ok(descriptor) => (descriptor.field_type, descriptor.index),
            Err(FieldDescriptorError::InvalidFieldDescriptor(
                FieldTypeError::InvalidWireType(_) | FieldTypeError::UnsupportedGroupType(_),
            )) => {
                // Handle invalid wire type by treating it as unparsed data
                return Err(ParseProtoError::MalformedProtobuf(data[idx..].to_vec()));
            }
        };

        let tag = match field_type {
            FieldType::Varint => {
                // Check if we have enough data for varint value
                if idx + tag_varint_len > data.len() {
                    return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
                }

                let var_int_len = find_varint_length(&data[idx + tag_varint_len..]);
                if idx + tag_varint_len + var_int_len > data.len() {
                    return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
                }

                skip_until = idx + tag_varint_len + var_int_len;
                Field {
                    tag: FieldType::Varint,
                    index,
                    value: FieldValue::Varint(parse_varint(
                        &data[idx + tag_varint_len..idx + tag_varint_len + var_int_len],
                    )),
                }
            }
            FieldType::I64 => {
                if idx + tag_varint_len + 8 > data.len() {
                    return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
                }
                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 8] = data[idx + tag_varint_len..idx + tag_varint_len + 8]
                    .try_into()
                    .unwrap();
                skip_until = idx + tag_varint_len + 8;
                Field {
                    tag: FieldType::I64,
                    index,
                    value: FieldValue::I64(parse_fixed64(&bytes) as isize),
                }
            }
            FieldType::Len => {
                let repeated_length = find_repeated_length(&data[idx + tag_varint_len..]);
                skip_until =
                    idx + tag_varint_len + repeated_length.skip_bytes + repeated_length.length;

                if skip_until > data.len() {
                    return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
                }

                let len_data = &data[idx + tag_varint_len + repeated_length.skip_bytes
                    ..idx + tag_varint_len + repeated_length.skip_bytes + repeated_length.length];

                // PANIC: for loop guarantees index exists when used without modification
                let tag_varint_len = find_varint_length(len_data);
                // Protobuf tag field can't be a signed interger
                let tag_bytes = parse_varint(&data[..tag_varint_len]) as usize;

                // Check if the first byte of the len data is a valid Protobuf Field Tag as a proxy
                // if the bytes should be parsed as a submessage.
                let test_is_sub_message = FieldDescriptor::try_from(&tag_bytes);

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
            FieldType::I32 => {
                // Check if we have enough data for 4-byte fixed32
                if idx + tag_varint_len + 4 > data.len() {
                    return Err(ParseProtoError::IncompleteData(ret, data[idx..].len()));
                }

                // Convert slice to array and parse as fixed32, then convert to isize
                let bytes: [u8; 4] = data[idx + tag_varint_len..idx + tag_varint_len + 4]
                    .try_into()
                    .unwrap();
                skip_until = idx + tag_varint_len + 4;
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
            value: FieldValue::LenSubmessage(vec![Field {
                tag: FieldType::Varint,
                index: 1,
                value: FieldValue::Varint(150),
            }]),
        }];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_varint_tag() {
        let parsed = parse_proto(&[0x80, 0x08, 0x01]).unwrap();
        let expected = vec![Field {
            tag: FieldType::Varint,
            index: 128,
            value: FieldValue::Varint(1),
        }];
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_incomplete_data_error() {
        // Test incomplete data at end of buffer
        let result = parse_proto(&[0x0d, 0x01, 0x00, 0x00]); // Incomplete I32 (needs 5 bytes, only 4 provided)
        match result {
            Err(ParseProtoError::IncompleteData(fields, remaining)) => {
                assert_eq!(fields.len(), 0);
                assert_eq!(remaining, 4); // 4 bytes remaining
            }
            _ => panic!("Expected IncompleteData error"),
        }

        // Test partial success with incomplete data
        let result = parse_proto(&[0x08, 0x01, 0x0d, 0x01, 0x00, 0x00]); // Complete varint, incomplete I32
        match result {
            Err(ParseProtoError::IncompleteData(fields, remaining)) => {
                assert_eq!(fields.len(), 1); // One field parsed successfully
                assert_eq!(remaining, 4); // 4 bytes remaining (incomplete I32)
            }
            _ => panic!("Expected IncompleteData error with partial success"),
        }
    }

    #[test]
    fn test_malformed_protobuf_error() {
        // Test invalid wire type 6
        let result = parse_proto(&[0x0F, 0x01]); // wire type 6, field 1
        match result {
            Err(ParseProtoError::MalformedProtobuf(bytes)) => {
                assert_eq!(bytes, vec![0x0F, 0x01]);
            }
            _ => panic!("Expected MalformedProtobuf error for wire type 6"),
        }

        // Test partial success with malformed data
        let result = parse_proto(&[0x08, 0x01, 0x0F, 0x02]); // Valid varint, then invalid wire type
        match result {
            Err(ParseProtoError::MalformedProtobuf(bytes)) => {
                assert_eq!(bytes, vec![0x0F, 0x02]);
            }
            _ => panic!("Expected MalformedProtobuf error with partial success"),
        }
    }
}
