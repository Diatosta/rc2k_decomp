//! If a path ends in `.png` or `.avi` and there
//! is no such file, changes the ending to `.pcx` or `.mpg`, in place. Paths of fewer than five
//! characters are left alone, and so is every other ending (also `.pcx`, `.mpg` and upper case).
//! Refuses what `text_length` and `file_exists` refuse.

use crate::errors::BadInput;
use crate::files::file_exists::file_exists;
use crate::session::TextPlace;

/// The path at `at`, its ending swapped there when the file is missing: the text as it is
/// afterwards.
pub fn swap_missing_extension(at: &dyn TextPlace) -> Result<Vec<u8>, BadInput> {
    let mut text = at.text("a path")?;
    if text.len() < 5 {
        return Ok(text);
    }
    let Some(partner) = rc2k_core::files::paths::missing_partner(&text) else {
        return Ok(text);
    };
    if !file_exists(&text)? {
        let n = text.len() - 4;
        at.write_text_at(n, partner, "a path's ending")?;
        text[n..].copy_from_slice(partner);
    }
    Ok(text)
}
