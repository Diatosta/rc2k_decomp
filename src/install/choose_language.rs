//! `install::choose_language` (0x41397A): makes one of the languages of `lang.cfg` the
//! language of the paths.
//!
//! Takes the language's number, from 1.
//! Nothing if the number is above the count of languages (`language_count`). A language is a
//! slot of 0x80 bytes at 0x51BEEE holding two texts, each ended by a NUL (`name:directory`
//! in the file); the second becomes what `%` stands for in a path.
//! Callers: `install::read_languages`, `install::install_from_command`, `install::language_page`.
//! Refuses number 0 (the original takes the slot before the first) and a slot whose two
//! texts do not end inside it.

use crate::errors::BadInput;
use crate::place::BytePlace;
use rc2k_core::install::config;

// ---- game data ----

pub const LANGUAGE_SLOT: usize = config::SLOT;
pub const LANGUAGE_SLOTS: usize = config::SLOTS;

pub fn choose_language(
    install: &'static crate::install::owner::Install,
    number: u32,
) -> Result<(), BadInput> {
    {
        // Converted: `rc2k_core::install::config` (`language_slot`, `language_directory`).
        let Some(slot) = config::language_slot(number, install.names.language_count.get())? else {
            return Ok(());
        };
        let slot = BytePlace::from(&install.names.languages)
            .add((slot * LANGUAGE_SLOT) as u32)
            .read(LANGUAGE_SLOT, "a language's slot")?;
        let directory = config::language_directory(&slot)?;
        let to = crate::files::state::locale_directory_place();
        to.write_bytes(directory, "the locale directory")?;
        to.offset(directory.len() as u32)
            .write(0u8, "the locale directory")?;
        Ok(())
    }
}
