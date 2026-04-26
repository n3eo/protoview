use std::fmt::Display;

pub use field::{Field, FieldValue};
pub use fixed::{i32_to_f32, i64_to_f64};
pub use proto_message::parse_proto;

mod field;
mod fixed;
pub mod proto_message;
mod repeated;
mod tag;
mod varint;

pub struct FieldList<'a>(pub Vec<Field<'a>>);

impl Display for FieldList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if let Some(first) = self.0.first() {
            write!(f, "{}", first)?;
        }
        for field in self.0.iter().skip(1) {
            write!(f, ", {}", field)?;
        }
        write!(f, "]")
    }
}
