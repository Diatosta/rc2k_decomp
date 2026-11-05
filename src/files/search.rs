//! The game's file search on std (phase 11; it was `FindFirstFileA` / `FindNextFileA` /
//! `FindClose`): the folder listed with `std::fs::read_dir`, its names matched as Windows
//! matched them (`rc2k_core::files::wildcard`) in NTFS's order. One search at a time, as
//! the game kept one (`hFindFile`, now 1 while a search is open). Each name found is put in
//! the game's search record (`FindFileData`) where the game reads it: the attributes at
//! +0, the name at +0x2C. `.` and `..` are not listed (the game passes over them by name).

use crate::files::state::{FIND_DATA, FIND_HANDLE};
use std::sync::Mutex;

/// FILE_ATTRIBUTE_DIRECTORY, _READONLY, _ARCHIVE: what the record's attributes hold.
pub const DIRECTORY: u32 = 0x10;
const READ_ONLY: u32 = 0x01;
const ARCHIVE: u32 = 0x20;

/// FindFileData.cFileName's offset and room (MAX_PATH).
const NAME_AT: usize = 0x2C;
const NAME_ROOM: usize = 260;
/// The record's length (`WIN32_FIND_DATAA`).
const RECORD_LEN: usize = 0x13E;

const NO_SEARCH: u32 = u32::MAX;
const OPEN: u32 = 1;

struct Search {
    found: Vec<(Vec<u8>, u32)>,
    next: usize,
}

static SEARCH: Mutex<Option<Search>> = Mutex::new(None);

/// The names in the folder of `pattern` (the game's bytes) that match its last part, with
/// their attributes, in NTFS's order; None when the folder cannot be listed or nothing
/// matches (the search fails, as `FindFirstFileA` did).
fn list(pattern: &[u8]) -> Option<Vec<(Vec<u8>, u32)>> {
    let cut = pattern.iter().rposition(|&b| b == b'\\' || b == b'/');
    let (folder, wanted) = match cut {
        Some(n) => (&pattern[..=n], &pattern[n + 1..]),
        None => (&b"."[..], pattern),
    };
    let mut names = Vec::new();
    for entry in std::fs::read_dir(crate::files::host_path(folder))
        .ok()?
        .flatten()
    {
        let name = crate::files::ansi_bytes(&entry.file_name().to_string_lossy());
        if rc2k_core::files::wildcard::matches(wanted, &name) {
            names.push(name);
        }
    }
    rc2k_core::files::wildcard::sort_names(&mut names);
    let found: Vec<_> = names
        .into_iter()
        .map(|name| {
            let mut path = folder.to_vec();
            path.extend(&name);
            let attributes =
                std::fs::metadata(crate::files::host_path(&path)).map_or(ARCHIVE, |m| {
                    let kind = if m.is_dir() { DIRECTORY } else { ARCHIVE };
                    kind | if m.permissions().readonly() {
                        READ_ONLY
                    } else {
                        0
                    }
                });
            (name, attributes)
        })
        .collect();
    (!found.is_empty()).then_some(found)
}

/// One found name put in the game's record.
fn put((name, attributes): &(Vec<u8>, u32)) {
    let mut record = [0u8; RECORD_LEN];
    record[..4].copy_from_slice(&attributes.to_le_bytes());
    let len = name.len().min(NAME_ROOM - 1);
    record[NAME_AT..NAME_AT + len].copy_from_slice(&name[..len]);
    for (n, &b) in record.iter().enumerate() {
        FIND_DATA.set(n, b);
    }
}

/// The search's next name not of the `skipped` attributes, put in the record; false when
/// there is none (the search stays open, as the game closes it).
pub fn next(skipped: u32) -> bool {
    let mut search = SEARCH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(s) = search.as_mut() else {
        return false;
    };
    while let Some(found) = s.found.get(s.next) {
        s.next += 1;
        put(found);
        if found.1 & skipped == 0 {
            return true;
        }
    }
    false
}

/// A search of `pattern` begun (any one open closed first): its first name not of the
/// `skipped` attributes, put in the record; false when there is none.
pub fn first(pattern: &[u8], skipped: u32) -> bool {
    let found = list(pattern);
    let open = found.is_some();
    *SEARCH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        found.map(|found| Search { found, next: 0 });
    FIND_HANDLE.set(if open { OPEN } else { NO_SEARCH });
    open && next(skipped)
}

/// The search closed, if there is one.
pub fn close() {
    *SEARCH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    FIND_HANDLE.set(NO_SEARCH);
}

/// The record's name (up to its NUL).
pub fn found_name() -> Vec<u8> {
    let name: Vec<u8> = (NAME_AT..NAME_AT + NAME_ROOM)
        .map(|n| FIND_DATA.get(n))
        .collect();
    name[..name.iter().position(|&b| b == 0).unwrap_or(NAME_ROOM)].to_vec()
}

/// The record's attributes.
pub fn found_attributes() -> u32 {
    FIND_DATA.get_u32(0)
}

/// Where the record's name is (a game address: the installer appends it by address).
pub fn found_name_place() -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&FIND_DATA).offset(NAME_AT as u32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_folder_is_searched_in_order_with_folders_passed_over() {
        let _lock = crate::store::test_lock();
        // The record's place, put back at the end.
        let kept: [u8; 0x140] = std::array::from_fn(|n| super::FIND_DATA.get(n));
        let dir = std::env::temp_dir().join(format!("rc2k_search_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub.pcx")).unwrap();
        for name in ["b.PCX", "a.pcx", "c.txt", "7.pcx"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let mut pattern = crate::files::ansi_bytes(&dir.to_string_lossy());
        pattern.extend(b"\\???.pcx");
        let mut seen = Vec::new();
        let mut more = super::first(&pattern, super::DIRECTORY);
        while more {
            seen.push(super::found_name());
            more = super::next(super::DIRECTORY);
        }
        super::close();
        assert_eq!(
            seen,
            [b"7.pcx".to_vec(), b"a.pcx".to_vec(), b"b.PCX".to_vec()]
        );
        let mut none = pattern.clone();
        none.extend(b"x");
        assert!(!super::first(&none, 0));
        super::close();
        for (n, &b) in kept.iter().enumerate() {
            super::FIND_DATA.set(n, b);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
