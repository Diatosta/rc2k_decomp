//! The texts of the pages this game adds (`rc2k_core::strings::own`): the file of the language
//! the game loaded, `files\%locale\<name>.rc2k.toml` with `<name>` the first `.loc` file's name,
//! read after the language file. A missing file, or one that is not UTF-8 TOML of texts (a
//! translator's slip), leaves English, with the reason logged; keys the game does not know are
//! logged once and ignored.

use crate::errors::BadInput;
use rc2k_core::strings::own::File;

/// The ending of a language's file of own texts, after the name of its `.loc` file.
pub const ENDING: &[u8] = b".rc2k.toml";

const LOCALE_PATTERN: &[u8] = b"files\\%locale\\*.loc";

/// The path of the own texts' file for the language `.loc` file `locale_file` (a file name) is
/// of, in the locale folder.
pub fn file_path(locale_file: &[u8]) -> Vec<u8> {
    let stem_end = locale_file
        .iter()
        .rposition(|&b| b == b'.')
        .unwrap_or(locale_file.len());
    let mut path = b"files\\%locale\\".to_vec();
    path.extend_from_slice(&locale_file[..stem_end]);
    path.extend_from_slice(ENDING);
    path
}

/// The name of the first `.loc` file of the locale folder, the one the language was loaded from.
fn locale_file_name() -> Result<Option<Vec<u8>>, BadInput> {
    let pattern = crate::files::resolve_path::resolve(LOCALE_PATTERN)?;
    let found = crate::files::search::first(&pattern, crate::files::search::DIRECTORY)
        .then(crate::files::search::found_name);
    crate::files::search::close();
    Ok(found)
}

/// Reads the language's own texts and keeps them for `own_text`.
pub fn load(strings: &crate::strings::owner::Strings) -> Result<(), BadInput> {
    let log = crate::logfile::program_log::log;
    let file = match locale_file_name()? {
        None => None,
        Some(name) => {
            let path = file_path(&name);
            let host = crate::files::host_path(&crate::files::resolve_path::resolve(&path)?);
            let read = std::fs::read(&host).map_err(|error| error.to_string());
            let parsed = read.and_then(|bytes| {
                let text = String::from_utf8(bytes).map_err(|_| "not UTF-8 text".to_string())?;
                File::parse(&text).map_err(|reason| reason.to_string())
            });
            match parsed {
                Ok(file) => {
                    for key in &file.unknown {
                        log(&format!(
                            "strings::own_texts: {}: unknown key {key}, ignored",
                            host.display()
                        ));
                    }
                    Some(file)
                }
                Err(error) => {
                    log(&format!(
                        "strings::own_texts: {}: {error}; the texts are in English",
                        host.display()
                    ));
                    None
                }
            }
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
    fn the_file_is_named_after_the_locale_file() {
        assert_eq!(
            file_path(b"PORTUGES.LOC"),
            b"files\\%locale\\PORTUGES.rc2k.toml".to_vec()
        );
        assert_eq!(
            file_path(b"english"),
            b"files\\%locale\\english.rc2k.toml".to_vec()
        );
    }
}
