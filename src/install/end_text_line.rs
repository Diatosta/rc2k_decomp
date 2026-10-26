//! `install::end_text_line` (0x415304; IDA had it as code outside any function, made one
//! 2026-10-19): a line feed and a NUL at ebx (`strings::append_lf`, `append_nul`), ebx past
//! them. In five of the installer's tables (0x51D7B4 ..); the installer's Rust driver does it
//! inline (`installer.rs`).
//!
//! In:  ebx = the cursor
//! Out: ebx + 2, the rest kept
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

/// The Rust version: the cursor after the two bytes.
pub fn end_text_line(cursor: u32) -> u32 {
    crate::install::installer::wb(cursor, b'\n');
    crate::install::installer::wb(cursor + 1, 0);
    cursor + 2
}
