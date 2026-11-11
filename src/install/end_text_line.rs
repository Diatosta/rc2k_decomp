//! `install::end_text_line` (0x415304): a line feed and a NUL at the cursor
//! (`strings::append_lf`, `append_nul`), the cursor past them. In five of the installer's tables (0x51D7B4 ..); the installer's Rust driver does it
//! inline (`installer.rs`).
//!
//! Takes the cursor.
//!
//! The two bytes are pushed on the text built.

/// The two bytes added to `out`.
pub fn end_text_line(out: &mut Vec<u8>) {
    out.extend_from_slice(b"\n\0");
}
