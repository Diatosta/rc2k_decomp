//! Reads the languages file `lang.cfg`: a settings text with the keys `lang1` .. `lang9`, each
//! value `name:directory`. Every key found has its value copied to its slot of 0x80 bytes; the
//! slots that start with a character above a space other than `:` and have a `:` are kept, moved
//! together to the front, the `:` made a NUL, and counted in `language_count`; then
//! `choose_language` for the first of them. Without the file nothing changes. Called by `start`
//! and `install::run_installer`. Refuses a value of 0x80 bytes or more.

use crate::errors::BadInput;
use crate::install::choose_language::{LANGUAGE_SLOT, LANGUAGE_SLOTS, choose_language};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

pub fn read_languages(
    install: &'static crate::install::owner::Install,
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<(), BadInput> {
    {
        let Some(text) = read_text(system, path)? else {
            return Ok(());
        };
        // Found first, stored after.
        let found = config::find_languages(&text, &case_table())?;
        let all = &install.names.languages;
        let mut slots = all.read_at(0, LANGUAGE_SLOT * LANGUAGE_SLOTS, "the languages' slots")?;
        for (slot, value) in found {
            let at = slot * LANGUAGE_SLOT;
            slots[at..at + value.len()].copy_from_slice(&value);
            slots[at + value.len()] = 0;
        }
        all.write_at(0, &slots, "the languages' slots")?;
        let kept = config::keep_languages(&mut slots);
        install.names.language_count.set(kept);
        choose_language(install, 1)
    }
}
