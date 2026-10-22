//! `files::append_file_name` (0x403040): a path's file name, after its last `/`, `\` or
//! `:`, appended at the cursor (no NUL). The search goes back from the NUL and stops
//! before the path's first byte (as `files::append_directory`'s): without a separator
//! after it the whole path is appended.
//!
//! In:  edx = the path, ebx = the cursor
//! Out: ebx past what was appended; eax, ecx, edx kept; the flags of `test cl, cl` on the
//!      NUL (zero set, carry clear)
//!
//! One caller, `install::run_after_exit`. Refuses (before anything) a null path.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::append_directory::text_of;

/// The Rust version: the cursor after.
pub unsafe fn append_file_name(path: *const u8, cursor: *mut u8) -> Result<*mut u8, BadInput> {
    if path.is_null() {
        return Err(BadInput("a null path"));
    }
    // Converted: `rc2k_core::files::paths::file_name`.
    let name = rc2k_core::files::paths::file_name(unsafe { text_of(path) });
    unsafe {
        std::ptr::copy(name.as_ptr(), cursor, name.len());
        Ok(cursor.add(name.len()))
    }
}
