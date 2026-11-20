//! Applies what the installation's config readers return: the record's directories and media
//! mode, the edition, the language list and the language's directory.

use crate::errors::BadInput;
use crate::install::choose_language::{LANGUAGE_SLOT, LANGUAGE_SLOTS, choose_language};
use rc2k_core::install::config;

/// Sets base directory A, the locale directory and the media mode the record names.
pub(super) fn apply_record(record: &config::Record) -> Result<(), BadInput> {
    for (directory, text) in [(0, &record.base), (1, &record.locale)] {
        if let Some(text) = text {
            crate::files::state::write_directory(directory, text, false, "an installer path")?;
        }
    }
    if let Some(mode) = record.mode {
        crate::files::state::MEDIA_MODE.set(mode);
    }
    Ok(())
}

/// Sets the edition's markers; what the file does not give keeps its value.
pub fn apply_sku(install: &crate::install::owner::Install, sku: &config::Sku) {
    if sku.oem {
        install.sku.oem.set(1u32);
    }
    if sku.dvd {
        install.sku.dvd.set(1u32);
    }
    install.sku.number.set(sku.number);
    if let Some(second) = sku.second {
        install.sku.second.set(second);
    }
    if let Some(third) = sku.third {
        install.sku.third.set(third);
    }
}

/// Stores the languages found, keeps the usable slots at the front, counts them and chooses
/// the first.
pub(super) fn apply_languages(
    install: &'static crate::install::owner::Install,
    languages: &config::Languages,
) -> Result<(), BadInput> {
    let all = &install.names.languages;
    let mut slots = all.read_at(0, LANGUAGE_SLOT * LANGUAGE_SLOTS, "the languages' slots")?;
    for (slot, value) in &languages.found {
        let at = slot * LANGUAGE_SLOT;
        slots[at..at + value.len()].copy_from_slice(value);
        slots[at + value.len()] = 0;
    }
    all.write_at(0, &slots, "the languages' slots")?;
    let kept = config::keep_languages(&mut slots);
    install.names.language_count.set(kept);
    choose_language(install, 1)
}

/// Makes `directory` (with its NUL) what `%` stands for in a path.
pub(super) fn apply_locale(directory: &[u8]) -> Result<(), BadInput> {
    crate::files::state::write_directory(1, directory, true, "the locale directory")
}
