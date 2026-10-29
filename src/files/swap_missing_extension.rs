//! `files::swap_missing_extension` (0x4032DB): if a path ends in `.png` or `.avi` and there
//! is no such file, changes the ending to `.pcx` or `.mpg`, in place.
//!
//! In:  eax = path, which is written to
//! Out: every register preserved
//! Paths of fewer than five characters are left alone, and so is every other ending (also
//! `.pcx` and `.mpg`, and upper case). Whether the file is there is asked of
//! `files::file_exists`. Two callers: the picture loader and the film player `sub_4161CD`.
//! Refuses what `text_length` and `file_exists` refuse. The name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::file_exists::file_exists;
use crate::pointers::GamePtr;

// ---- Rust version ----

/// The path at `at`, its ending swapped there when the file is missing: the text as it is
/// afterwards.
pub fn swap_missing_extension(at: GamePtr) -> Result<Vec<u8>, BadInput> {
    let mut text = crate::files::cstring(at, "a path")?;
    if text.len() < 5 {
        return Ok(text);
    }
    // Converted: `rc2k_core::files::paths::missing_partner`.
    let Some(partner) = rc2k_core::files::paths::missing_partner(&text) else {
        return Ok(text);
    };
    if !file_exists(&text)? {
        let n = text.len() - 4;
        at.offset(n as u32).write(*partner, "a path's ending")?;
        text[n..].copy_from_slice(partner);
    }
    Ok(text)
}
