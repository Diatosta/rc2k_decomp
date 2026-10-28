//! `install::normalise_path` (0x4139EC): a directory's path in the installer's one form.
//!
//! In:  esi = the path, edx = where to write it
//! Out: carry clear if it was written, set if not; every register preserved
//! The form: lower case (through `g_lowerCase`), `\` for every `/`, and a `\` at the end.
//! An empty path stands for the current directory (`sub_4030D1`: `GetCurrentDirectoryA`
//! into the place, a `\` added). Otherwise it must start with `\\` or `//`, or with a
//! drive letter and `:`, after which a `\` is put if there is none; anything else is not
//! written. 14 callers, most of them in the installer.
//! Refuses a path of 0xF8 bytes or more, and a current directory that does not fit the
//! 0xFF bytes asked with.

use crate::errors::BadInput;
use crate::install::folders::{Folders, Native};

// ---- Rust version ----

/// `sub_4030D1`'s asking: the current directory, or None if it cannot be had.
pub fn current_directory() -> Result<Option<Vec<u8>>, BadInput> {
    // The game's place for it is 0x100 bytes, 0xFF asked for.
    match crate::os::current_folder() {
        None => Ok(None),
        Some(folder) if folder.len() >= 0xFF => {
            Err(BadInput("a current directory longer than the place for it"))
        }
        Some(folder) => Ok(Some(folder)),
    }
}

/// The path as it is written, with its NUL; None if it is not one that is taken.
/// `current` is the current directory, asked for only if the path is empty.
pub fn normalised(
    path: *const u8,
    current: &mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>,
) -> Result<Option<Vec<u8>>, BadInput> {
    let path = crate::files::cstring(
        crate::pointers::GamePtr::new(crate::pointers::game_address(path)),
        "a path",
    )?;
    // The system's rules (`install::folders`; Windows: `rc2k_core::install::paths::normalise`).
    <Native as Folders>::normalised(&path, current)
}

/// For Rust callers: writes the path at `to`. Returns whether it was written.
pub fn normalise_path(path: *const u8, to: *mut u8) -> Result<bool, BadInput> {
    let Some(text) = normalised(path, &mut || current_directory())? else {
        return Ok(false);
    };
    crate::pointers::GamePtr::new(crate::pointers::game_address(to))
        .write_bytes(&text, "a normalised path")?;
    Ok(true)
}
