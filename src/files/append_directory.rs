//! `files::append_directory` (0x403070): a path's directory, up to and with its last `/`,
//! `\` or `:`, appended at the cursor (no NUL). The search goes back from the NUL and stops
//! before the path's first byte: a separator there alone is not found, and a path without
//! one appends nothing.
//!
//! In:  edx = the path, ebx = the cursor
//! Out: ebx past what was appended; eax, ecx, edx kept; the flags of the last `cmp eax,
//!      edx`: carry set (and zero clear) after a copy, zero set (carry clear) without one
//!
//! Five callers in the installer and `install::run_after_exit`. `files::append_file_name`
//! is its other half. Refuses (before anything) a null path.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

/// The Rust version: the cursor after, and whether a directory was found.
pub fn append_directory(path: *const u8, cursor: *mut u8) -> Result<(*mut u8, bool), BadInput> {
    if path.is_null() {
        return Err(BadInput("a null path"));
    }
    let text = crate::files::cstring(GamePtr::new(path as u32), "a path")?;
    let Some(directory) = rc2k_core::files::paths::directory(&text) else {
        return Ok((cursor, false));
    };
    GamePtr::new(cursor as u32).write_bytes(directory, "a path being built")?;
    Ok((cursor.wrapping_add(directory.len()), true))
}
