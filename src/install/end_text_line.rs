//! A line feed and a NUL pushed on the text being built (an entry of five of the installer's
//! tables).

/// The two bytes added to `out`.
pub fn end_text_line(out: &mut Vec<u8>) {
    out.extend_from_slice(b"\n\0");
}
