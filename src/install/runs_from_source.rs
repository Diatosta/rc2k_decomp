//! `install::runs_from_source` (0x412DA3): whether the game was started from the
//! directory the registry names as its installation's source.
//!
//! In:  nothing
//! Out: zero set if so; every register preserved
//! `read_registry`; then, only if the status is 1 and a source is named and the current
//! directory can be had (`files::current_directory` into `byte_94E8B0`), the two are
//! compared with `strings::same_text`. Everything else is "no" (zero and carry clear).
//! Callers: the main loop `sub_415383` on every start, and `install::check_clio_disc`.

use crate::errors::BadInput;
use crate::install::outside::Outside;
use crate::install::vars::INSTALLER_NAMES;
use crate::strings::same_text::compare_texts;
use std::cmp::Ordering;

/// byte_94E8B0: the scratch path (also `files::numbered_path`'s buffer).
pub fn scratch_path() -> *mut u8 {
    crate::files::numbered_path::buffer()
}

// ---- Rust version ----

/// None if the question does not arise, else how the source stands to the current
/// directory (`Equal`: started from there).
pub unsafe fn source_against_directory(
    outside: &mut dyn Outside,
) -> Result<Option<Ordering>, BadInput> {
    unsafe {
        outside.read_registry();
        if INSTALLER_NAMES.status.get() != 1 || INSTALLER_NAMES.source.get(0) == 0 {
            return Ok(None);
        }
        let directory = scratch_path();
        if !outside.current_directory(directory)? {
            return Ok(None);
        }
        Ok(Some(compare_texts(
            crate::pointers::host(INSTALLER_NAMES.source.addr()) as *const u8,
            directory,
        )?))
    }
}

pub unsafe fn runs_from_source(outside: &mut dyn Outside) -> Result<bool, BadInput> {
    Ok(unsafe { source_against_directory(outside) }? == Some(Ordering::Equal))
}
