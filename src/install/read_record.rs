//! Reads the installation's record `var\ral.zog`: a settings text with three keys, each taken if
//! it is there: `cdrom` becomes base directory A, `installation` the media mode (held to
//! 0 .. 3) and `language` what `%` stands for in a path. The two texts get a `\` after them
//! unless they are empty or end in `\` or `/`. Refuses a text of 0xFE bytes or more.

use crate::errors::BadInput;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

/// The record, or `None` without the file; `install::apply_config::apply_record` applies it.
pub fn read_record(
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<Option<config::Record>, BadInput> {
    let Some(text) = read_text(system, path)? else {
        return Ok(None);
    };
    Ok(Some(config::read_record(&text, &case_table())?))
}

/// Reads the record and applies it; nothing changes without the file.
pub fn load_record(system: &crate::system::owner::System, path: &[u8]) -> Result<(), BadInput> {
    match read_record(system, path)? {
        Some(record) => crate::install::apply_config::apply_record(&record),
        None => Ok(()),
    }
}
