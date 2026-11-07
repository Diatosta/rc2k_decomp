//! `install::choose_language` (0x41397A): makes one of the languages of `lang.cfg` the
//! language of the paths.
//!
//! In:  eax = the language's number, from 1
//! Out: every register preserved
//! Nothing if the number is above the count of languages (`dword_51BEEA`). A language is a
//! slot of 0x80 bytes at 0x51BEEE holding two texts, each ended by a NUL (`name:directory`
//! in the file); the second becomes what `%` stands for in a path (`byte_518AA8`).
//! Callers: `install::read_languages`, `sub_412F34`, `sub_414199`.
//! Refuses number 0 (the original takes the slot before the first) and a slot whose two
//! texts do not end inside it.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::BadInput;
use crate::place::Place;
use rc2k_core::install::config;

// ---- game data ----

pub const LANGUAGE_SLOT: usize = config::SLOT;
pub const LANGUAGE_SLOTS: usize = config::SLOTS;

// ---- Rust version ----

pub fn choose_language(
    install: &'static crate::install::owner::Install,
    number: u32,
) -> Result<(), BadInput> {
    {
        // Converted: `rc2k_core::install::config` (`language_slot`, `language_directory`).
        let Some(slot) = config::language_slot(number, install.names.language_count.get())? else {
            return Ok(());
        };
        let slot = Place::from(&install.names.languages)
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
