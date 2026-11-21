//! What a folder or a zip holds of the game, for the first-run page: which kind of folder it
//! is, where the game's folder is inside what was unpacked, and which names of a zip are safe
//! to write.

/// A file only the game's data has, as a path under the game's folder (the CD's data folder
/// and the installed copy both hold it).
pub const MARKER: &str = "files/d/gfx/jade.bun";
/// The files that say a folder is an installation, not the download / CD layout.
pub(crate) const INSTALLED_MARKS: [&str; 2] = ["var/ral.zog", "var/install.cfg"];

/// What a folder holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The game's data and its installation record: the game runs in it as it is.
    Installed,
    /// The game's data as downloaded or on the CD: the installer runs on it.
    Source,
    /// Not the game's folder.
    Nothing,
}

/// The kind of a folder, `has` saying whether a path under it (with `/`) exists.
pub fn classify(has: impl Fn(&str) -> bool) -> Kind {
    if !has(MARKER) {
        Kind::Nothing
    } else if INSTALLED_MARKS.iter().any(|m| has(m)) {
        Kind::Installed
    } else {
        Kind::Source
    }
}

#[cfg(test)]
/// The folder among `candidates` (paths under the unpacked zip, `""` its root, shallowest
/// first) that holds the game, with its kind: the first that is not `Nothing`.
pub(crate) fn find_game_folder<'a>(
    candidates: &[&'a str],
    has: impl Fn(&str, &str) -> bool,
) -> Option<(&'a str, Kind)> {
    candidates
        .iter()
        .find_map(|folder| match classify(|path| has(folder, path)) {
            Kind::Nothing => None,
            kind => Some((*folder, kind)),
        })
}

/// A zip entry's name as a relative path of safe parts, or none when it would leave the folder
/// it is unpacked into (a `..`, a root, a drive letter) or names nothing. `\` counts as `/`.
pub fn safe_parts(name: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    for part in name.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => return None,
            p if p.contains(':') || p.contains('\0') => return None,
            p => parts.push(p),
        }
    }
    if parts.is_empty() || name.starts_with(['/', '\\']) {
        None
    } else {
        Some(parts)
    }
}

/// One file of a zip: its name, how it is stored and where its data starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// 0 stored, 8 deflated.
    pub method: u16,
    pub packed: u64,
    pub size: u64,
    /// The offset of the entry's local header.
    pub header: u64,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The end of a zip's central directory in the file's last bytes `tail`: where the directory
/// starts and how many entries it has. Zip64 archives are refused by their markers.
pub fn directory_of(tail: &[u8]) -> Result<(u64, usize), &'static str> {
    let at = (0..tail.len().saturating_sub(21))
        .rev()
        .find(|&i| tail[i..].starts_with(b"PK\x05\x06"))
        .ok_or("this is not a zip file")?;
    let count = u16_at(tail, at + 10).ok_or("the zip's end is cut short")?;
    let offset = u32_at(tail, at + 16).ok_or("the zip's end is cut short")?;
    if count == 0xFFFF || offset == 0xFFFF_FFFF {
        return Err("zips over 4 GB (zip64) are not read; unpack it and pick the folder");
    }
    Ok((u64::from(offset), usize::from(count)))
}

/// The entries of a central directory `directory` (its `count` entries, from its start).
pub fn entries_of(directory: &[u8], count: usize) -> Result<Vec<Entry>, &'static str> {
    let cut = "the zip's directory is cut short";
    let mut entries = Vec::with_capacity(count);
    let mut at = 0;
    for _ in 0..count {
        if directory.get(at..at + 4) != Some(b"PK\x01\x02") {
            return Err("the zip's directory is damaged");
        }
        let flags = u16_at(directory, at + 8).ok_or(cut)?;
        let method = u16_at(directory, at + 10).ok_or(cut)?;
        let packed = u32_at(directory, at + 20).ok_or(cut)?;
        let size = u32_at(directory, at + 24).ok_or(cut)?;
        let name_len = usize::from(u16_at(directory, at + 28).ok_or(cut)?);
        let extra = usize::from(u16_at(directory, at + 30).ok_or(cut)?);
        let comment = usize::from(u16_at(directory, at + 32).ok_or(cut)?);
        let header = u32_at(directory, at + 42).ok_or(cut)?;
        let name = directory.get(at + 46..at + 46 + name_len).ok_or(cut)?;
        if flags & 1 != 0 {
            return Err("an encrypted zip is not read");
        }
        entries.push(Entry {
            name: String::from_utf8_lossy(name).into_owned(),
            method,
            packed: u64::from(packed),
            size: u64::from(size),
            header: u64::from(header),
        });
        at += 46 + name_len + extra + comment;
    }
    Ok(entries)
}

/// Where an entry's data starts, from its local header's 30 first bytes.
pub fn data_start(entry: &Entry, local: &[u8]) -> Result<u64, &'static str> {
    if local.get(0..4) != Some(b"PK\x03\x04") {
        return Err("the zip's entry is damaged");
    }
    let name = u64::from(u16_at(local, 26).ok_or("the zip's entry is damaged")?);
    let extra = u64::from(u16_at(local, 28).ok_or("the zip's entry is damaged")?);
    Ok(entry.header + 30 + name + extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_are_told_apart_by_their_marks() {
        let held = |files: &'static [&'static str]| move |p: &str| files.contains(&p);
        assert_eq!(classify(held(&[])), Kind::Nothing);
        assert_eq!(classify(held(&[MARKER])), Kind::Source);
        assert_eq!(
            classify(held(&[MARKER, "var/install.cfg"])),
            Kind::Installed
        );
        assert_eq!(classify(held(&[MARKER, "var/ral.zog"])), Kind::Installed);
        assert_eq!(classify(held(&["var/ral.zog"])), Kind::Nothing);
    }

    #[test]
    fn the_game_folder_is_the_first_that_holds_it() {
        let has = |folder: &str, path: &str| folder == "Rally" && path == MARKER;
        assert_eq!(
            find_game_folder(&["", "Rally"], has),
            Some(("Rally", Kind::Source))
        );
        assert_eq!(find_game_folder(&["", "Other"], has), None);
    }

    #[test]
    fn unsafe_names_are_refused() {
        assert_eq!(
            safe_parts("files\\d/x.bun"),
            Some(vec!["files", "d", "x.bun"])
        );
        assert_eq!(safe_parts("./a//b"), Some(vec!["a", "b"]));
        for bad in [
            "../x",
            "a/../../x",
            "/etc/x",
            "\\x",
            "c:/x",
            "a/c:x",
            "",
            "./",
        ] {
            assert_eq!(safe_parts(bad), None, "{bad}");
        }
    }

    /// A zip with one stored file, built by hand.
    fn tiny_zip() -> Vec<u8> {
        let mut z = Vec::new();
        z.extend_from_slice(b"PK\x03\x04");
        z.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        z.extend_from_slice(&[0; 4]); // crc
        z.extend_from_slice(&3u32.to_le_bytes());
        z.extend_from_slice(&3u32.to_le_bytes());
        z.extend_from_slice(&1u16.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes());
        z.extend_from_slice(b"a");
        z.extend_from_slice(b"xyz");
        let directory = z.len() as u32;
        z.extend_from_slice(b"PK\x01\x02");
        z.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        z.extend_from_slice(&[0; 4]);
        z.extend_from_slice(&3u32.to_le_bytes());
        z.extend_from_slice(&3u32.to_le_bytes());
        z.extend_from_slice(&1u16.to_le_bytes());
        z.extend_from_slice(&[0; 12]);
        z.extend_from_slice(&0u32.to_le_bytes());
        z.extend_from_slice(b"a");
        let size = z.len() as u32 - directory;
        z.extend_from_slice(b"PK\x05\x06");
        z.extend_from_slice(&[0; 4]);
        z.extend_from_slice(&1u16.to_le_bytes());
        z.extend_from_slice(&1u16.to_le_bytes());
        z.extend_from_slice(&size.to_le_bytes());
        z.extend_from_slice(&directory.to_le_bytes());
        z.extend_from_slice(&[0; 2]);
        z
    }

    #[test]
    fn a_zips_directory_is_read() {
        let z = tiny_zip();
        let (offset, count) = directory_of(&z).unwrap();
        assert_eq!(count, 1);
        let entries = entries_of(&z[offset as usize..], count).unwrap();
        assert_eq!(entries[0].name, "a");
        assert_eq!((entries[0].size, entries[0].method), (3, 0));
        assert_eq!(data_start(&entries[0], &z).unwrap(), 31);
        assert!(directory_of(b"not a zip at all, no end record here").is_err());
    }
}
