//! Pretty-printing of decoded protobuf [`FieldList`]s with independently
//! configurable colors and indentation.
//!
//! Both properties are plain *data* handed to a single printing routine, so
//! every combination of {colored, plain} x {indented, flat} works without the
//! printing code ever branching on either:
//!
//! * **Colors** are a [`ColorScheme`]: a lookup table from a semantic
//!   [`Token`] to an ANSI code. [`NoColors`] maps everything to `""`, which
//!   is all that "uncolored" means.
//! * **Indentation** is an [`IndentWriter`]: a [`fmt::Write`] adapter that
//!   prefixes every new line with the padding. Empty padding makes it a
//!   pass-through, which is all that "not indented" means.
//!
//! The two compose freely through [`Styled`]:
//!
//! ```
//! use protoview_cli::display::Styled;
//! use protoview_lib::FieldList;
//! # let fields = FieldList(vec![]);
//! // plain & flat
//! println!("{}", Styled::plain(&fields));
//! // plain & indented
//! println!("{}", Styled::plain(&fields).indented("  "));
//! // colored & flat
//! println!("{}", Styled::plain(&fields).colored());
//! // colored & indented
//! println!("{}", Styled::plain(&fields).colored().indented("  "));
//! ```

use std::fmt::{self, Display, Write};

use protoview_lib::{Field, FieldList, FieldValue, i32_to_f32, i64_to_f64};
use zigzag_rs::ZigZag;

/// Semantic role of a piece of printed text.
///
/// The printing code emits tokens instead of hard-coding escape sequences;
/// the active [`ColorScheme`] decides what, if anything, they look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// The field type, e.g. `Varint` or `SubMessage`.
    Type,
    /// The field index, e.g. the `2` in `SubMessage @ 2:`.
    Index,
    /// Varint values.
    Int,
    /// Fixed-size (`I32`/`I64`) values.
    Fixed,
    /// `Len` values that are valid UTF-8.
    Str,
    /// `Len` values that are not valid UTF-8.
    Bytes,
    /// Text that is never styled (e.g. legacy `SEGroup` bodies).
    Plain,
}

/// Maps semantic [`Token`]s to ANSI SGR color codes.
///
/// Returning `""` leaves a token unstyled, which is how monochrome output is
/// expressed: the printing code never needs to know whether colors are on.
/// New schemes (e.g. light themes) are just new tables.
pub trait ColorScheme {
    /// ANSI SGR parameter for `token`, or `""` to print it unstyled.
    fn code(&self, token: Token) -> &str;
}

/// The classic protoview color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiColors;

impl ColorScheme for AnsiColors {
    fn code(&self, token: Token) -> &str {
        match token {
            Token::Type => "35",  // magenta
            Token::Index => "34", // blue
            Token::Int => "33",   // yellow
            Token::Fixed => "36", // cyan
            Token::Str => "32",   // green
            Token::Bytes => "31", // red
            Token::Plain => "",
        }
    }
}

/// The scheme that styles nothing; used for plain output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoColors;

impl ColorScheme for NoColors {
    fn code(&self, _token: Token) -> &str {
        ""
    }
}

/// A [`fmt::Write`] adapter that indents every line.
///
/// It fully owns the indentation, so the printing code only ever writes
/// newlines and enters/leaves nesting levels via [`push`](Self::push) and
/// [`pop`](Self::pop). Empty padding turns it into a pass-through, which is
/// how "not indented" is expressed.
struct IndentWriter<'a, W: fmt::Write> {
    inner: W,
    padding: &'a str,
    depth: usize,
    /// A line break was written and the next non-empty line still needs its
    /// padding.
    pending: bool,
}

impl<'a, W: fmt::Write> IndentWriter<'a, W> {
    fn new(inner: W, padding: &'a str) -> Self {
        Self {
            inner,
            padding,
            depth: 0,
            pending: true,
        }
    }

    /// Enters one indentation level.
    fn push(&mut self) {
        self.depth += 1;
    }

    /// Leaves one indentation level.
    fn pop(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Writes one line's worth of content, inserting the padding first if
    /// the line does not start with one already.
    fn write_line(&mut self, line: &str) -> fmt::Result {
        if self.pending && !line.is_empty() {
            self.inner.write_str(&self.padding.repeat(self.depth))?;
            self.pending = false;
        }
        self.inner.write_str(line)
    }
}

impl<W: fmt::Write> fmt::Write for IndentWriter<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // No padding configured: indentation is off, pass everything through.
        if self.padding.is_empty() {
            return self.inner.write_str(s);
        }

        // `write!`/`writeln!` feed this adapter in arbitrary chunks, so the
        // newline handling must work at every possible chunk boundary.
        let mut lines = s.split('\n');
        if let Some(first) = lines.next() {
            self.write_line(first)?;
        }
        for line in lines {
            self.inner.write_str("\n")?;
            self.pending = true;
            self.write_line(line)?;
        }
        Ok(())
    }
}

/// Prints a field tree, consulting `colors` for every [`Token`] and
/// delegating all line placement to [`IndentWriter`].
///
/// This is the only place that knows the layout; because colors and
/// indentation are looked up rather than branched on, this single routine
/// serves every combination.
struct Printer<'a, C: ColorScheme, W: fmt::Write> {
    out: IndentWriter<'a, W>,
    colors: &'a C,
}

impl<C: ColorScheme, W: fmt::Write> Printer<'_, C, W> {
    /// Writes `content` styled as `token`.
    ///
    /// The single point where coloring happens: an empty code (as produced
    /// by [`NoColors`]) writes the content untouched.
    fn styled(&mut self, token: Token, content: impl Display) -> fmt::Result {
        match self.colors.code(token) {
            "" => write!(self.out, "{content}"),
            code => write!(self.out, "\x1b[{code}m{content}\x1b[0m"),
        }
    }

    /// Writes unstyled text.
    fn raw(&mut self, content: impl Display) -> fmt::Result {
        write!(self.out, "{content}")
    }

    /// Prints each field on its own line.
    fn field_list(&mut self, fields: &FieldList<'_>) -> fmt::Result {
        for field in &fields.0 {
            self.field(field)?;
            writeln!(self.out)?;
        }
        Ok(())
    }

    /// Prints a single field, recursing into submessages one indentation
    /// level deeper.
    fn field(&mut self, field: &Field<'_>) -> fmt::Result {
        self.styled(Token::Type, type_name(&field.value))?;
        self.raw(" @ ")?;
        self.styled(Token::Index, field.index)?;
        self.raw(": ")?;

        if let FieldValue::LenSubmessage(sub) = &field.value {
            self.raw("[")?;
            writeln!(self.out)?;
            self.out.push();
            self.field_list(sub)?;
            self.out.pop();
            self.raw("]")
        } else {
            self.styled(value_token(&field.value), ValueText(&field.value))
        }
    }
}

/// A [`FieldValue`] rendered as text, the presentation-side counterpart of
/// the parsing types in `protoview-lib`.
///
/// The orphan rule forbids implementing `Display` for the lib's types here,
/// so the rendering lives on this wrapper instead. Submessages never go
/// through it: the printer recurses into them to own their layout.
struct ValueText<'a, 'v>(&'v FieldValue<'a>);

impl Display for ValueText<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FieldValue::I64(value) => {
                write!(f, "sint {} | uint {} | double {:e}", value, *value as usize, i64_to_f64(*value as i64))
            }
            FieldValue::I32(value) => {
                write!(f, "sint {} | uint {} | float  {:e}", value, *value as u32, i32_to_f32(*value as i32))
            }
            FieldValue::Varint(value) => {
                write!(f, "int {} | uint {} | sint {}", value, *value as usize, i64::zigzag_decode(*value as u64))?;
                if let Some(b) = convert_to_if_bool(*value) {
                    write!(f, " | bool {}", b)?;
                }
                Ok(())
            }
            FieldValue::LenPrimitive(items) => write!(f, "{:?}", String::from_utf8(items.to_vec()).unwrap_or_else(|_| format!("{:?}", items))),
            // Only reachable for legacy `SEGroup`s.
            FieldValue::SEGroup(items) => write!(f, "SEGroup({:?})", items),
            FieldValue::LenSubmessage(_) => unreachable!("submessages are printed by the printer"),
        }
    }
}

/// Interprets a varint as a bool when it is exactly `0` or `1`.
fn convert_to_if_bool(inp: isize) -> Option<bool> {
    match inp {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

/// The name printed for a field's type.
///
/// Submessages are named after their value variant, not their wire type
/// (`SubMessage` instead of `Len`).
fn type_name(value: &FieldValue<'_>) -> &'static str {
    match value {
        FieldValue::Varint(_) => "Varint",
        FieldValue::I32(_) => "I32",
        FieldValue::I64(_) => "I64",
        FieldValue::LenPrimitive(_) => "Len",
        FieldValue::LenSubmessage(_) => "SubMessage",
        FieldValue::SEGroup(_) => "SEGroup",
    }
}

/// Chooses the [`Token`] a value is printed with.
///
/// The value's text itself comes from [`ValueText`], so this is the only
/// other CLI-side knowledge about values.
fn value_token(value: &FieldValue<'_>) -> Token {
    match value {
        FieldValue::Varint(_) => Token::Int,
        FieldValue::I32(_) | FieldValue::I64(_) => Token::Fixed,
        FieldValue::LenPrimitive(bytes) if std::str::from_utf8(bytes).is_ok() => Token::Str,
        FieldValue::LenPrimitive(_) => Token::Bytes,
        // Only reachable for legacy `SEGroup`s, which are never styled.
        _ => Token::Plain,
    }
}

/// A [`FieldList`] printed with a configurable [`ColorScheme`] and
/// indentation padding.
///
/// The two settings are independent, so all four combinations come from the
/// same printing routine:
///
/// ```
/// use protoview_cli::display::Styled;
/// use protoview_lib::FieldList;
/// # let fields = FieldList(vec![]);
/// // plain & flat
/// let a = Styled::plain(&fields);
/// // plain & indented
/// let b = Styled::plain(&fields).indented("  ");
/// // colored & flat
/// let c = Styled::plain(&fields).colored();
/// // colored & indented
/// let d = Styled::plain(&fields).colored().indented("  ");
/// # let _ = (a, b, c, d);
/// ```
#[derive(Debug, Clone)]
pub struct Styled<'f, 'a, C: ColorScheme = NoColors> {
    fields: &'f FieldList<'a>,
    colors: C,
    padding: &'f str,
}

impl<'f, 'a> Styled<'f, 'a, NoColors> {
    /// Starts from the most reduced form: uncolored and flat.
    pub fn plain(fields: &'f FieldList<'a>) -> Self {
        Self {
            fields,
            colors: NoColors,
            padding: "",
        }
    }
}

impl<'f, 'a, C: ColorScheme> Styled<'f, 'a, C> {
    /// Indents every nesting level by `padding`.
    ///
    /// Empty padding keeps the output flat; this is the only switch for
    /// indentation.
    #[must_use]
    pub fn indented(self, padding: &'f str) -> Self {
        Self { padding, ..self }
    }

    /// Switches to the classic protoview [`AnsiColors`] scheme.
    #[must_use]
    pub fn colored(self) -> Styled<'f, 'a, AnsiColors> {
        Styled {
            fields: self.fields,
            colors: AnsiColors,
            padding: self.padding,
        }
    }
}

impl<C: ColorScheme> Display for Styled<'_, '_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut printer = Printer {
            out: IndentWriter::new(f, self.padding),
            colors: &self.colors,
        };
        printer.field_list(self.fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protoview_lib::FieldType;

    /// Simple sink so [`IndentWriter`] can be tested directly.
    #[derive(Default)]
    struct Sink(String);

    impl fmt::Write for Sink {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.push_str(s);
            Ok(())
        }
    }

    fn fields() -> FieldList<'static> {
        FieldList(vec![
            Field {
                tag: FieldType::Len,
                index: 1,
                value: FieldValue::LenPrimitive(b"hello"),
            },
            Field {
                tag: FieldType::Len,
                index: 2,
                value: FieldValue::LenSubmessage(FieldList(vec![
                    Field {
                        tag: FieldType::Varint,
                        index: 1,
                        value: FieldValue::Varint(1),
                    },
                    Field {
                        tag: FieldType::Varint,
                        index: 2,
                        value: FieldValue::Varint(42),
                    },
                ])),
            },
        ])
    }

    /// Wraps a code and text in the expected ANSI escapes.
    fn ansi(code: &str, text: &str) -> String {
        format!("\x1b[{code}m{text}\x1b[0m")
    }

    /// Removes ANSI SGR sequences so colored and plain output can be
    /// compared byte for byte.
    fn strip_ansi(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    const PLAIN_FLAT: &str = concat!(
        "Len @ 1: \"hello\"\n",
        "SubMessage @ 2: [\n",
        "Varint @ 1: int 1 | uint 1 | sint -1 | bool true\n",
        "Varint @ 2: int 42 | uint 42 | sint 21\n",
        "]\n",
    );

    const PLAIN_INDENTED: &str = concat!(
        "Len @ 1: \"hello\"\n",
        "SubMessage @ 2: [\n",
        "  Varint @ 1: int 1 | uint 1 | sint -1 | bool true\n",
        "  Varint @ 2: int 42 | uint 42 | sint 21\n",
        "]\n",
    );

    #[test]
    fn plain_and_flat() {
        assert_eq!(Styled::plain(&fields()).to_string(), PLAIN_FLAT);
    }

    #[test]
    fn plain_and_indented() {
        assert_eq!(
            Styled::plain(&fields()).indented("  ").to_string(),
            PLAIN_INDENTED
        );
    }

    #[test]
    fn colored_and_flat() {
        let out = Styled::plain(&fields()).colored().to_string();
        let lines: Vec<&str> = out.lines().collect();

        assert_eq!(
            lines[0],
            format!(
                "{} @ {}: {}",
                ansi("35", "Len"),
                ansi("34", "1"),
                ansi("32", "\"hello\"")
            )
        );
        assert_eq!(
            lines[1],
            format!("{} @ {}: [", ansi("35", "SubMessage"), ansi("34", "2"))
        );
        assert_eq!(
            lines[2],
            format!(
                "{} @ {}: {}",
                ansi("35", "Varint"),
                ansi("34", "1"),
                ansi("33", "int 1 | uint 1 | sint -1 | bool true")
            )
        );
        assert_eq!(
            lines[3],
            format!(
                "{} @ {}: {}",
                ansi("35", "Varint"),
                ansi("34", "2"),
                ansi("33", "int 42 | uint 42 | sint 21")
            )
        );
        assert_eq!(lines[4], "]");
        assert_eq!(strip_ansi(&out), PLAIN_FLAT);
    }

    #[test]
    fn colored_and_indented() {
        let colored = Styled::plain(&fields())
            .colored()
            .indented("  ")
            .to_string();
        let plain = Styled::plain(&fields()).indented("  ").to_string();
        assert_eq!(strip_ansi(&colored), plain);

        // The colored scheme must not leak into the line structure.
        assert_eq!(colored.lines().count(), plain.lines().count());
    }

    #[test]
    fn colored_output_is_plain_output_plus_escapes() {
        // The property that makes the whole design work: colored and plain
        // share one printing routine, so stripping the escapes must yield
        // exactly the plain output, in both layouts.
        for (colored, plain) in [
            (
                Styled::plain(&fields()).colored().to_string(),
                Styled::plain(&fields()).to_string(),
            ),
            (
                Styled::plain(&fields())
                    .colored()
                    .indented("  ")
                    .to_string(),
                Styled::plain(&fields()).indented("  ").to_string(),
            ),
        ] {
            assert_eq!(strip_ansi(&colored), plain);
        }
    }

    #[test]
    fn deeply_nested_indentation() {
        let fields = FieldList(vec![Field {
            tag: FieldType::Len,
            index: 1,
            value: FieldValue::LenSubmessage(FieldList(vec![Field {
                tag: FieldType::Len,
                index: 1,
                value: FieldValue::LenSubmessage(FieldList(vec![Field {
                    tag: FieldType::Varint,
                    index: 1,
                    value: FieldValue::Varint(4),
                }])),
            }])),
        }]);

        assert_eq!(
            Styled::plain(&fields).indented("  ").to_string(),
            concat!(
                "SubMessage @ 1: [\n",
                "  SubMessage @ 1: [\n",
                "    Varint @ 1: int 4 | uint 4 | sint 2\n",
                "  ]\n",
                "]\n",
            )
        );
    }

    #[test]
    fn indent_writer_passes_through_without_padding() {
        let mut sink = Sink::default();
        let mut writer = IndentWriter::new(&mut sink, "");
        writer.push();
        writeln!(writer, "a\nb").unwrap();
        assert_eq!(sink.0, "a\nb\n");
    }

    #[test]
    fn indent_writer_indents_each_new_line() {
        let mut sink = Sink::default();
        {
            let mut writer = IndentWriter::new(&mut sink, "  ");
            write!(writer, "a").unwrap();
            writeln!(writer).unwrap();
            writer.push();
            write!(writer, "b\nc").unwrap();
            writer.pop();
            write!(writer, "\nd").unwrap();
        }
        assert_eq!(sink.0, "a\n  b\n  c\nd");
    }

    #[test]
    fn indent_writer_keeps_blank_lines_blank() {
        let mut sink = Sink::default();
        {
            let mut writer = IndentWriter::new(&mut sink, "  ");
            writer.push();
            write!(writer, "a\n\nb").unwrap();
        }
        assert_eq!(sink.0, "  a\n\n  b");
    }
}
