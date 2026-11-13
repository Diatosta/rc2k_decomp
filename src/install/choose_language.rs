//! Makes one of the languages of `lang.cfg` (number from 1) the language of the paths: the
//! second of the language's two NUL-ended texts (`name:directory`) is what `%` stands for in a
//! path. Nothing if the number is above the language count. Called by `install::read_languages`,
//! `install::install_from_command`, `install::language_page`. Refuses number 0 and a slot whose
//! two texts do not end inside it.

use crate::errors::BadInput;
use crate::place::BytePlace;
use rc2k_core::install::config;

pub const LANGUAGE_SLOT: usize = config::SLOT;
pub const LANGUAGE_SLOTS: usize = config::SLOTS;

pub fn choose_language(
    install: &'static crate::install::owner::Install,
    number: u32,
) -> Result<(), BadInput> {
    {
        let Some(slot) = config::language_slot(number, install.names.language_count.get())? else {
            return Ok(());
        };
        let slot = BytePlace::from(&install.names.languages)
            .add((slot * LANGUAGE_SLOT) as u32)
            .read(LANGUAGE_SLOT, "a language's slot")?;
        let directory = config::language_directory(&slot)?;
        crate::files::state::write_directory(1, directory, true, "the locale directory")?;
        Ok(())
    }
}
