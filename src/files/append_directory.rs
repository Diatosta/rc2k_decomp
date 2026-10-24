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

/// The bytes of a NUL-terminated text, without the NUL (an unbounded scan, as the
/// original's).
pub unsafe fn text_of<'a>(text: *const u8) -> &'a [u8] {
    unsafe { std::ffi::CStr::from_ptr(text.cast()).to_bytes() }
}

/// The Rust version: the cursor after, and whether a directory was found.
pub unsafe fn append_directory(
    path: *const u8,
    cursor: *mut u8,
) -> Result<(*mut u8, bool), BadInput> {
    if path.is_null() {
        return Err(BadInput("a null path"));
    }
    // Converted: `rc2k_core::files::paths::directory`.
    let Some(directory) = rc2k_core::files::paths::directory(unsafe { text_of(path) }) else {
        return Ok((cursor, false));
    };
    unsafe {
        std::ptr::copy(directory.as_ptr(), cursor, directory.len());
        Ok((cursor.add(directory.len()), true))
    }
}
