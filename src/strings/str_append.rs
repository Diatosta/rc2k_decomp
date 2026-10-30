//! `strings::str_append` (0x401F7D): copies a NUL-terminated string to the cursor, without the NUL.
//!
//! In:  eax = source string, ebx = cursor
//! Out: ebx = new cursor

use crate::errors::BadInput;
use crate::pointers::GamePtr;

// A loop target lies inside the first 5 bytes, so the original runs from a private copy.
// ---- Rust version ----

/// The text at `src` copied to `cursor` without its NUL; the cursor after it.
pub fn str_append(src: GamePtr, cursor: GamePtr) -> Result<GamePtr, BadInput> {
    let len = crate::strings::text_length::nul_at(src.addr(), "a text to append")?;
    src.copy_to(cursor, len as usize, "a text to append")?;
    Ok(cursor.offset(len))
}
