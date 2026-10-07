pub use field::{Field, FieldType, FieldValue};
pub use fixed::{i32_to_f32, i64_to_f64};
pub use proto_message::parse_proto;
pub use proto_message::ParseProtoError;

mod field;
mod fixed;
pub mod proto_message;
mod repeated;
mod tag;
mod varint;

#[derive(Debug, PartialEq, Eq)]
pub struct FieldList<'a>(pub Vec<Field<'a>>);
