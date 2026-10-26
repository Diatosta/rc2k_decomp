//! `strings::str_append` (0x401F7D): copies a NUL-terminated string to the cursor, without the NUL.
//!
//! In:  eax = source string, ebx = cursor
//! Out: ebx = new cursor

use crate::errors::BadInput;
use crate::pointers::GamePtr;

// A loop target lies inside the first 5 bytes, so the original runs from a private copy.
// ---- Rust version ----

/// The text at `src` copied to `cursor` without its NUL; the cursor after it.
pub fn str_append(src: *const u8, cursor: *mut u8) -> Result<*mut u8, BadInput> {
    let len = crate::strings::text_length::nul_at(
        crate::pointers::game_address(src),
        "a text to append",
    )?;
    GamePtr::new(crate::pointers::game_address(src)).copy_to(
        GamePtr::new(crate::pointers::game_address(cursor)),
        len as usize,
        "a text to append",
    )?;
    Ok(cursor.wrapping_add(len as usize))
}
