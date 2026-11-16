//! Copies the own texts' file of the installed language (`strings::own_texts`) from the
//! program's `locale` folder, in the download folder, into the installed language folder, where
//! the game reads it. The language is the first `.loc` file of the installed folder, as at
//! start. Nothing is copied, and that is logged, when the download folder has no such file (the
//! game then shows English).

/// The folder in the download folder that holds the files of every language.
pub const SOURCE_FOLDER: &[u8] = b"locale\\";

/// The first `.loc` file name in `names`, in name order, whatever its case.
fn first_locale_file(mut names: Vec<Vec<u8>>) -> Option<Vec<u8>> {
    names.sort();
    names
        .into_iter()
        .find(|name| name.len() > 4 && name[name.len() - 4..].eq_ignore_ascii_case(b".loc"))
}

/// `source` is the download folder and `installed` the new folder (each with its `\` at the
/// end), `percent` what `%` stands for in a path.
pub fn copy_own_texts(source: &[u8], installed: &[u8], percent: &[u8]) {
    let log = crate::logfile::program_log::log;
    let mut folder = installed.to_vec();
    folder.extend_from_slice(b"files\\%locale\\");
    let folder: Vec<u8> = folder
        .iter()
        .flat_map(|&b| if b == b'%' { percent.to_vec() } else { vec![b] })
        .collect();
    let Ok(entries) = std::fs::read_dir(crate::files::host_path(&folder)) else {
        return;
    };
    let names = entries
        .flatten()
        .map(|e| crate::files::ansi_bytes(&e.file_name().to_string_lossy()))
        .collect();
    let Some(locale_file) = first_locale_file(names) else {
        return;
    };
    let name = crate::strings::own_texts::file_path(&locale_file);
    let name = &name[b"files\\%locale\\".len()..];
    let mut from = source.to_vec();
    from.extend_from_slice(SOURCE_FOLDER);
    from.extend_from_slice(name);
    let mut to = folder;
    to.extend_from_slice(name);
    let (from, to) = (crate::files::host_path(&from), crate::files::host_path(&to));
    match std::fs::copy(&from, &to) {
        Ok(_) => log(&format!("install: copied {}", from.display())),
        Err(error) => log(&format!(
            "install: no own texts for the language ({}: {error}); English will show",
            from.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_loc_file_by_name() {
        let names = vec![
            b"PORTUGES.TXT".to_vec(),
            b"b.LOC".to_vec(),
            b"A.loc".to_vec(),
            b".loc".to_vec(),
        ];
        assert_eq!(first_locale_file(names), Some(b"A.loc".to_vec()));
        assert_eq!(first_locale_file(vec![b"x.txt".to_vec()]), None);
    }
}
