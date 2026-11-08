//! `install::end_text_line` (0x415304; IDA had it as code outside any function, made one
//! 2026-10-19): a line feed and a NUL at ebx (`strings::append_lf`, `append_nul`), ebx past
//! them. In five of the installer's tables (0x51D7B4 ..); the installer's Rust driver does it
//! inline (`installer.rs`).
//!
//! Takes the cursor.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments. Texts as values (2026-11-07):
//! the two bytes pushed on the text built.

/// The Rust version: the two bytes added to `out`.
pub fn end_text_line(out: &mut Vec<u8>) {
    out.extend_from_slice(b"\n\0");
}
