//! Reads the languages file `lang.cfg`: a settings text with the keys `lang1` .. `lang9`, each
//! value `name:directory`. Refuses a value of 0x80 bytes or more.

use crate::errors::BadInput;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

/// The languages the file names, or `None` without the file;
/// `install::apply_config::apply_languages` applies them.
pub fn read_languages(
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<Option<config::Languages>, BadInput> {
    let Some(text) = read_text(system, path)? else {
        return Ok(None);
    };
    Ok(Some(config::read_languages(&text, &case_table())?))
}

/// Reads the languages and applies them; nothing changes without the file.
pub fn load_languages(
    install: &'static crate::install::owner::Install,
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<(), BadInput> {
    match read_languages(system, path)? {
        Some(languages) => crate::install::apply_config::apply_languages(install, &languages),
        None => Ok(()),
    }
}
