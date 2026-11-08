//! `install::normalise_path` (0x4139EC): a directory's path in the installer's one form.
//!
//! Takes the path, where to write it.
//! The form: lower case (through `g_lowerCase`), `\` for every `/`, and a `\` at the end.
//! An empty path stands for the current directory (0x4030D1: `GetCurrentDirectoryA`
//! into the place, a `\` added). Otherwise it must start with `\\` or `//`, or with a
//! drive letter and `:`, after which a `\` is put if there is none; anything else is not
//! written. 14 callers, most of them in the installer.
//! Refuses a path of 0xF8 bytes or more, and a current directory that does not fit the
//! 0xFF bytes asked with.

use crate::errors::BadInput;
use crate::install::folders::{Folders, Native};

// ---- Rust version ----

/// 0x4030D1's asking: the current directory, or None if it cannot be had.
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
    path: crate::place::Place,
    current: &mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>,
) -> Result<Option<Vec<u8>>, BadInput> {
    let path = path.text("a path")?;
    // The system's rules (`install::folders`; Windows: `rc2k_core::install::paths::normalise`).
    <Native as Folders>::normalised(&path, current)
}

/// `normalised` of a path in hand (up to its NUL or its end).
pub fn normalised_bytes(
    path: &[u8],
    current: &mut dyn FnMut() -> Result<Option<Vec<u8>>, BadInput>,
) -> Result<Option<Vec<u8>>, BadInput> {
    let n = path.iter().position(|&c| c == 0).unwrap_or(path.len());
    <Native as Folders>::normalised(&path[..n], current)
}

/// For Rust callers: writes the path at `to`. Returns whether it was written.
pub fn normalise_path(
    path: crate::place::Place,
    to: crate::place::Place,
) -> Result<bool, BadInput> {
    let Some(text) = normalised(path, &mut || current_directory())? else {
        return Ok(false);
    };
    // An empty path takes the current directory, of any length; the places written hold 0x100.
    to.game_ptr()
        .unbounded()
        .limited(0x100)
        .write_bytes(&text, "a normalised path")?;
    Ok(true)
}
