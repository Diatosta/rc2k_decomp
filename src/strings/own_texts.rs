//! The texts of the pages this game adds (`rc2k_core::strings::own`): the language is the one of
//! the first `.loc` file of `files\%locale`, as `load_language` takes it, and its texts are the
//! ones built into the program (`assets/locale/<name>.rc2k.toml`). The game's folder is never
//! read for them: a copy of the folder from an older install would show new texts in English,
//! and a stale file there is ignored. A language with no built-in file shows English.

use crate::errors::BadInput;

const LOCALE_PATTERN: &[u8] = b"files\\%locale\\*.loc";

/// The language's name: a `.loc` file name without its ending.
fn language_name(locale_file: &[u8]) -> String {
    let stem_end = locale_file
        .iter()
        .rposition(|&b| b == b'.')
        .unwrap_or(locale_file.len());
    String::from_utf8_lossy(&locale_file[..stem_end]).into_owned()
}

/// The name of the first `.loc` file of the locale folder, the one the language was loaded from.
fn locale_file_name() -> Result<Option<Vec<u8>>, BadInput> {
    let pattern = crate::files::resolve_path::resolve(LOCALE_PATTERN)?;
    let found = crate::files::search::first(&pattern, crate::files::search::DIRECTORY)
        .then(crate::files::search::found_name);
    crate::files::search::close();
    Ok(found)
}

/// Picks the built-in own texts of the installed language for `own_text`.
pub fn load(strings: &crate::strings::owner::Strings) -> Result<(), BadInput> {
    let file = match locale_file_name()? {
        None => None,
        Some(name) => {
            let language = language_name(&name);
            let file = rc2k_core::strings::own::for_language(&language);
            if file.is_none() {
                crate::program_log::log(&format!(
                    "strings::own_texts: no built-in texts for the language {language}; \
                     the texts are in English"
                ));
            }
            file
        }
    };
    *strings
        .language
        .own
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = file;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_language_is_the_locale_file_without_its_ending() {
        assert_eq!(language_name(b"PORTUGES.LOC"), "PORTUGES");
        assert_eq!(language_name(b"english"), "english");
    }
}
