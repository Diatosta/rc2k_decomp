//! `install::check_folder_name` (0x413A6C): whether a folder path (from its third byte on,
//! past a drive letter and colon) holds a character that is not allowed in a folder name
//! (`unk_51DE18`: `"`, `+`, `*`, `,`, `;`, `=`, `/`, `:`, `<`, `>`, `?`, `\`, `{`, `}`,
//! `[`, `]`, `|`, 0x7F) or a double backslash.
//!
//! In:  edx the path
//! Out: carry set if a bad character or `\\` is found, clear otherwise; every register kept
//! Caller `install::install_files`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::bridge::{Raw, read_through};

/// Whether `path` has a bad character or `\\` (the original's carry).
pub fn check_folder_name(path: u32) -> bool {
    // Converted: `rc2k_core::install::paths::bad_folder_name` (the characters, RAL.EXE's
    // `unk_51DE18`, as its constant).
    let path = unsafe { Raw::new(path as *const u8) };
    read_through(rc2k_core::install::paths::bad_folder_name(&path))
}
