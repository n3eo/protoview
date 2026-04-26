use std::{
    fmt::{self, Display},
    rc::Rc,
};

use protoview_lib::{Field, FieldList, FieldValue, i32_to_f32};

use crate::colored_display::{Colorer, NoColor};

pub trait Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn indent(&self) -> Self;
}

pub struct Indenter<'a, T> {
    depth: usize,
    padding: &'a str,
    colorer: Rc<dyn Colorer<T> + 'a>,
}

impl<'a, T: Display> Indent for Indenter<'a, T> {
    fn indent(&self) -> Self {
        Self {
            depth: self.depth + 1,
            padding: self.padding,
            colorer: self.colorer.clone(),
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.padding.repeat(self.depth))
    }
}

impl<'a, T> Indenter<'a, T> {
    pub fn new(padding: &'a str, colorer: impl Colorer<T> + 'a) -> Self {
        Self {
            depth: 0,
            padding,
            colorer: std::rc::Rc::new(colorer),
        }
    }

    pub fn color(&self, f: &mut std::fmt::Formatter<'_>, item: &T) -> std::fmt::Result {
        self.colorer.color(f, item)
    }
}

pub trait IndentedDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, i: &'a impl Indent) -> fmt::Result;
}

impl<'a> IndentedDisplay<'a> for FieldList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, i: &'a impl Indent) -> fmt::Result {
        for field in &self.0 {
            IndentedDisplay::fmt(field, f, i)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<'a> IndentedDisplay<'a> for Field<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>, indenter: &'a impl Indent) -> fmt::Result {
        indenter.fmt(formatter)?;
        match &self.value {
            FieldValue::LenSubmessage(fields) => {
                writeln!(formatter, "SubMessage @ {}: [", self.index)?;
                let new_indenter = indenter.indent();
                IndentedDisplay::fmt(fields, formatter, &new_indenter)?;
                indenter.fmt(formatter)?;
                write!(formatter, "]")
            }
            _ => {
                write!(formatter, "{}", self)
            }
        }
    }
}

// pub trait FieldDisplayExt<'a> {
//     /// Display the field with optional indentation
//     fn display_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result;

//     /// Display the field with optional indentation and colors
//     fn display_with_colors(
//         &self,
//         f: &mut fmt::Formatter<'_>,
//         indent: &str,
//         use_colors: bool,
//     ) -> fmt::Result;
// }

// impl<'a> FieldDisplayExt<'a> for Field<'a> {
//     fn display_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
//         self.display_with_colors(f, indent, false)
//     }

//     fn display_with_colors(
//         &self,
//         f: &mut fmt::Formatter<'_>,
//         indent: &str,
//         use_colors: bool,
//     ) -> fmt::Result {
//         if use_colors {
//             write!(f, "\x1b[34m{}\x1b[0m", self.index)?; // Blue for field index
//         } else {
//             write!(f, "{}", self.index)?;
//         }

//         match &self.value {
//             FieldValue::LenSubmessage(fields) => {
//                 if use_colors {
//                     writeln!(f, ": \x1b[35mSubMessage\x1b[0m = \x1b[34m{{")?; // Magenta for type, Blue for braces
//                 } else {
//                     writeln!(f, ": SubMessage = {{")?;
//                 }
//                 let new_indent = format!("{}{}", indent, "    ");
//                 for field in fields.iter() {
//                     write!(f, "{}    ", indent)?;
//                     field.display_with_colors(f, &new_indent, use_colors)?;
//                 }
//                 if use_colors {
//                     writeln!(f, "{}\x1b[34m}}\x1b[0m", indent)?; // Blue for closing brace
//                 } else {
//                     writeln!(f, "{}}}", indent)?;
//                 }
//             }
//             FieldValue::I32(value) | FieldValue::I64(value) => {
//                 let new_indent = format!("{}{}", indent, "    ");
//                 if use_colors {
//                     writeln!(
//                         f,
//                         ": \x1b[35m{}\x1b[0m = \n{}\x1b[33mint\x1b[0m   : {}\n{}\x1b[33mfloat\x1b[0m : {}",
//                         self.tag,
//                         new_indent,
//                         value,
//                         new_indent,
//                         i32_to_f32(*value as i32)
//                     )?;
//                 } else {
//                     writeln!(
//                         f,
//                         ": {} = \n{}int   : {}\n{}float : {}",
//                         self.tag,
//                         new_indent,
//                         value,
//                         new_indent,
//                         i32_to_f32(*value as i32)
//                     )?;
//                 }
//             }
//             FieldValue::Varint(value) => {
//                 let new_indent = format!("{}{}", indent, "    ");
//                 if use_colors {
//                     write!(
//                         f,
//                         ": \x1b[35m{}\x1b[0m = \n{}\x1b[33msigned\x1b[0m   : {}\n{}\x1b[33munsigned\x1b[0m : {}",
//                         self.tag, new_indent, value, new_indent, *value as usize
//                     )?;
//                     match *value {
//                         0 => writeln!(
//                             f,
//                             "\n{}\x1b[33mbool\x1b[0m     : \x1b[31mfalse\x1b[0m",
//                             new_indent
//                         )?, // Red for false
//                         1 => writeln!(
//                             f,
//                             "\n{}\x1b[33mbool\x1b[0m     : \x1b[32mtrue\x1b[0m",
//                             new_indent
//                         )?, // Green for true
//                         _ => writeln!(f, "")?,
//                     };
//                 } else {
//                     write!(
//                         f,
//                         ": {} = \n{}signed   : {}\n{}unsigned : {}",
//                         self.tag, new_indent, value, new_indent, *value as usize
//                     )?;
//                     match *value {
//                         0 => writeln!(f, "\n{}bool     : false", new_indent)?,
//                         1 => writeln!(f, "\n{}bool     : true", new_indent)?,
//                         _ => writeln!(f, "")?,
//                     };
//                 }
//             }
//             _ => {
//                 if use_colors {
//                     write!(f, ": \x1b[35m{}\x1b[0m = ", self.tag)?; // Magenta for field type
//                     write!(f, "{}", self.value)?; // Use default value display
//                     writeln!(f, "")?;
//                 } else {
//                     writeln!(f, ": {} = {}", self.tag, self.value)?;
//                 }
//             }
//         }
//         Ok(())
//     }
// }

// /// Wrapper struct for displaying a vector of fields
// pub struct FieldList<'a>(pub Vec<Field<'a>>);

// impl<'a> fmt::Display for FieldList<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for field in &self.0 {
//             write!(f, "{}", field)?;
//         }
//         Ok(())
//     }
// }

// /// Wrapper struct for displaying a vector of fields with colors
// pub struct ColoredFieldList<'a> {
//     pub fields: Vec<Field<'a>>,
//     pub use_colors: bool,
// }

// impl<'a> fmt::Display for ColoredFieldList<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for field in &self.fields {
//             field.display_with_colors(f, "", self.use_colors)?;
//         }
//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn test_display_implementation() {
//         // Test FieldType display
//         assert_eq!(format!("{}", FieldType::Varint), "Varint");
//         assert_eq!(format!("{}", FieldType::I64), "I64");
//         assert_eq!(format!("{}", FieldType::Len), "Len");
//         assert_eq!(format!("{}", FieldType::I32), "I32");

//         // Test FieldValue display
//         assert_eq!(format!("{}", FieldValue::Varint(42)), "42");
//         assert_eq!(
//             format!("{}", FieldValue::I32(123)),
//             "123, 0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000061"
//         );
//         assert_eq!(
//             format!("{}", FieldValue::I32(1065353216)),
//             "1065353216, 0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005263544247"
//         );
//         assert_eq!(
//             format!("{}", FieldValue::I64(456)),
//             "456, 0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002253"
//         );
//         assert_eq!(
//             format!("{}", FieldValue::LenPrimitive(b"hello")),
//             "\"hello\""
//         );
//         assert_eq!(
//             format!("{}", FieldValue::LenPrimitive(&[0xFF, 0xFE])),
//             "[255, 254]"
//         );

//         // Test Field display
//         let field = Field {
//             tag: FieldType::Varint,
//             index: 1,
//             value: FieldValue::Varint(42),
//         };
//         assert_eq!(
//             format!("{}", field),
//             "1: Varint = \n    signed   : 42\n    unsigned : 42\n"
//         );

//         let string_field = Field {
//             tag: FieldType::Len,
//             index: 2,
//             value: FieldValue::LenPrimitive(b"test string"),
//         };
//         assert_eq!(format!("{}", string_field), "2: Len = \"test string\"\n");
//     }
// }
