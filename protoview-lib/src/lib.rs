mod proto_parsing;
mod schema_parsing;

pub use proto_parsing::proto_message::{ParseProtoError, parse_proto};
pub use proto_parsing::{i32_to_f32, i64_to_f64};
