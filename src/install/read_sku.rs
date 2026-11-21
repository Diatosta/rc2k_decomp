//! Reads the edition file `files\d\gfx\sku`: a settings text with the key `sku`. Its value may
//! start with the marks `oem:` and `dvd:`, in any order and number; then a number, after the next
//! `:` a second byte and after the next `.` within 254 characters a third byte.

use crate::errors::BadInput;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

/// The edition the file names, or `None` without the file or its key. Nothing of the game reads
/// the edition after this (its markers were only ever written), so the caller keeps just the
/// refusals reading and parsing the file can give.
pub fn read_sku(
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<Option<config::Sku>, BadInput> {
    let Some(text) = read_text(system, path)? else {
        return Ok(None);
    };
    Ok(config::read_sku(&text, &case_table())?)
}

/// The edition file's path.
pub const SKU_PATH: &[u8] = b"files\\d\\gfx\\sku";
