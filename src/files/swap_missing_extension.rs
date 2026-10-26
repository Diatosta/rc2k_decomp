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
use crate::strings::text_length::text_length;

// ---- Rust version ----

/// Where the path's last four characters are, if it has at least five.
unsafe fn ending(path: *mut u8) -> Result<Option<*mut [u8; 4]>, BadInput> {
    let length = unsafe { text_length(path) }? as usize;
    Ok((length >= 5).then(|| path.wrapping_add(length - 4).cast()))
}

pub unsafe fn swap_missing_extension(path: *mut u8) -> Result<(), BadInput> {
    unsafe {
        let Some(ending) = ending(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::files::paths::missing_partner`.
        let length = text_length(path)? as usize;
        let Some(partner) =
            rc2k_core::files::paths::missing_partner(std::slice::from_raw_parts(path, length))
        else {
            return Ok(());
        };
        if !file_exists(path)? {
            ending.write(*partner);
        }
        Ok(())
    }
}
