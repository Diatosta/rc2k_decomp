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

pub fn swap_missing_extension(path: *mut u8) -> Result<(), BadInput> {
    let at = GamePtr::new(crate::pointers::game_address(path));
    let text = crate::files::cstring(at, "a path")?;
    if text.len() < 5 {
        return Ok(());
    }
    // Converted: `rc2k_core::files::paths::missing_partner`.
    let Some(partner) = rc2k_core::files::paths::missing_partner(&text) else {
        return Ok(());
    };
    if !file_exists(&text)? {
        at.offset(text.len() as u32 - 4)
            .write(*partner, "a path's ending")?;
    }
    Ok(())
}
