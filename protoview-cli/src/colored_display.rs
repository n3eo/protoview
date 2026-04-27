use std::fmt::{self, Display};

use protoview_lib::{Field, FieldValue};

// Implement such that there are two (later more) variants how to color the output
// one without color, the other with color
pub trait Colorer<T> {
    fn color(&self, f: &mut fmt::Formatter<'_>, item: &T) -> fmt::Result;
}

pub struct NoColor {}

impl<T: Display> Colorer<T> for NoColor {
    fn color(&self, f: &mut fmt::Formatter<'_>, item: &T) -> fmt::Result {
        write!(f, "{}", item)
    }
}
