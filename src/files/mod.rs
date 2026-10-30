pub mod append_directory;
pub mod close_stream;
pub mod copy_file;
pub mod directories;
pub mod drive_free_space_kb;
pub mod drive_type;
pub mod file_exists;
pub mod file_size;
pub mod handle;
pub mod is_cd_drive;
pub mod numbered_path;
pub mod open_stream;
pub mod read_file;
pub mod read_first_match;
pub mod read_level_file;
pub mod read_packed_file;
pub mod read_stream_block;
pub mod read_stream_byte;
pub mod read_whole_file;
pub mod resolve_path;
pub mod scratch;
pub mod search;
pub mod stream;
pub mod swap_missing_extension;
pub mod write_file;

pub mod write_stream_block;
pub mod write_stream_byte;

/// Windows-1252's characters for 0x80 .. 0x9F (the rest of the code page is Latin-1).
const CP1252_HIGH: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// The game's bytes as text, read the way the Win32 `A` functions read them (the ANSI code
/// page, Windows-1252: phase 6, the file calls through `std`).
pub fn ansi_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH[(b - 0x80) as usize],
            _ => b as char,
        })
        .collect()
}

/// Text as the game's bytes (Windows-1252, `ansi_text`'s inverse); a character the code
/// page lacks becomes `?`, as Windows' conversion writes it.
pub fn ansi_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c as u32 {
            0..=0x7F | 0xA0..=0xFF => c as u8,
            _ => CP1252_HIGH
                .iter()
                .position(|&h| h == c)
                .map_or(b'?', |n| 0x80 + n as u8),
        })
        .collect()
}

/// The text at `at` up to its NUL (not included), read inside one allocation: a text with
/// no NUL before its allocation's end is refused.
pub fn cstring(
    at: crate::pointers::GamePtr,
    what: &str,
) -> Result<Vec<u8>, crate::errors::BadInput> {
    use rc2k_core::strings::bytes::Bytes;
    let bytes = at.bytes(what)?;
    let mut text = Vec::new();
    for n in 0.. {
        match bytes.byte(n) {
            Some(0) => return Ok(text),
            Some(b) => text.push(b),
            None => {
                return Err(crate::errors::BadInput(
                    "a text with no end in its allocation",
                ));
            }
        }
    }
    unreachable!()
}

/// A path in the game's bytes (no NUL) as a `PathBuf`: the one place a game path becomes
/// the system's.
#[cfg(windows)]
pub fn host_path(path: &[u8]) -> std::path::PathBuf {
    std::path::PathBuf::from(ansi_text(path))
}

/// A path in the game's bytes (no NUL) as a `PathBuf`: the one place a game path becomes
/// the system's. On Unix (phase 11) as Windows read it: `\` a separator as `/` is, a drive
/// letter (`c:`) the root, and each name found in its folder without regard to case (the
/// game's files are named in any case); a name that is not there (a file to be made) is
/// kept as written.
#[cfg(unix)]
pub fn host_path(path: &[u8]) -> std::path::PathBuf {
    use std::path::{Path, PathBuf};
    let text = ansi_text(path).replace('\\', "/");
    let bytes = text.as_bytes();
    let text = if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        format!("/{}", text[2..].trim_start_matches('/'))
    } else {
        text
    };
    let mut at = if text.starts_with('/') {
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };
    let found_in = |folder: &Path, name: &str| -> Option<std::ffi::OsString> {
        let folder = if folder.as_os_str().is_empty() {
            Path::new(".")
        } else {
            folder
        };
        std::fs::read_dir(folder).ok()?.flatten().find_map(|e| {
            let n = e.file_name();
            n.to_str()
                .is_some_and(|n| n.to_lowercase() == name.to_lowercase())
                .then_some(n)
        })
    };
    for name in text.split('/').filter(|n| !n.is_empty()) {
        let exact = at.join(name);
        if name == "." || name == ".." || exact.symlink_metadata().is_ok() {
            at = exact;
            continue;
        }
        at = match found_in(&at, name) {
            Some(real) => at.join(real),
            None => exact,
        };
    }
    if text.ends_with('/') {
        at.push("");
    }
    at
}

/// The NUL-terminated path at `at` (the game's ANSI bytes) as a `PathBuf`.
pub fn ansi_path(
    at: crate::pointers::GamePtr,
) -> Result<std::path::PathBuf, crate::errors::BadInput> {
    Ok(host_path(&cstring(at, "a path")?))
}

#[cfg(test)]
mod ansi_tests {
    #[test]
    fn ansi_text_is_windows_1252() {
        assert_eq!(
            super::ansi_text(b"files\\b\\tx1822.pcx"),
            "files\\b\\tx1822.pcx"
        );
        assert_eq!(super::ansi_text(&[0x80, 0xE7, 0xE3, 0x9C]), "€çãœ");
        assert_eq!(super::ansi_bytes("€çãœ"), [0x80, 0xE7, 0xE3, 0x9C]);
        assert_eq!(super::ansi_bytes("a\u{3B1}"), b"a?");
    }

    #[cfg(unix)]
    #[test]
    fn host_paths_are_found_without_regard_to_case() {
        let dir = std::env::temp_dir().join(format!("rc2k_paths_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Files/B")).unwrap();
        std::fs::write(dir.join("Files/B/TX1822.PCX"), b"x").unwrap();
        let game = |p: &str| super::ansi_bytes(&format!("{}\\{p}", dir.display()));
        assert_eq!(
            super::host_path(&game("files\\b\\tx1822.pcx")),
            dir.join("Files/B/TX1822.PCX")
        );
        assert_eq!(
            super::host_path(&game("files\\new.txt")),
            dir.join("Files/new.txt")
        );
        assert_eq!(
            super::host_path(b"c:\\tmp"),
            std::path::PathBuf::from("/tmp")
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// The file code's variables, out of the image since 2026-10-24 (phase 5; each has its own
/// entry in `moved::MOVED`).
pub mod state {
    use crate::moved::Place;
    use std::sync::atomic::AtomicU32;

    /// g_lastFilePath (0x4F52B0): the path of the last file opened or searched for, or
    /// none (the fatal-error report names it). The original holds the path's address (and
    /// the report reads what is there then); its text since 2026-10-29 (phase 13).
    static LAST_FILE_PATH: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

    /// `g_lastFilePath` set to `path` (without its NUL), or to none.
    pub fn set_last_file_path(path: Option<&[u8]>) {
        *LAST_FILE_PATH.lock().unwrap_or_else(|e| e.into_inner()) = path.map(<[u8]>::to_vec);
    }

    /// `g_lastFilePath`'s path.
    pub fn last_file_path() -> Option<Vec<u8>> {
        LAST_FILE_PATH
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    /// hFindFile (0x4E05A4): 1 while a search is open (`files::search`), `u32::MAX` for
    /// none.
    pub static FIND_HANDLE: AtomicU32 = AtomicU32::new(u32::MAX);
    /// FindFileData (0x4E05A8, `WIN32_FIND_DATAA`, 0x13E bytes; 0x140 here): the search's
    /// record (`files::search` fills its attributes and name).
    pub static FIND_DATA: Place<[u32; 80]> = Place::new([0; 80]);
    /// The attributes a search passes over (0x4E06E6, the dword after the record): the
    /// directory bit; the installer clears it for its walk of folders.
    pub static SKIPPED_ATTRIBUTES: AtomicU32 = AtomicU32::new(0x10);
    /// dword_60B2CC (g_readBytes): the bytes `read_whole_file` read.
    pub static BYTES_READ: AtomicU32 = AtomicU32::new(0);
    /// g_mediaMode (0x5189A4): how the game is installed (0 .. 3), as the installer's record
    /// says (`var\ral.zog`); `resolve_path` looks at it.
    pub static MEDIA_MODE: AtomicU32 = AtomicU32::new(0);
    /// g_baseDirectory (0x5189A8, 0x100 bytes) and, after it, g_localeDirectory (0x518AA8,
    /// 0x100 bytes): strings, empty at the start. One block, as in the image.
    pub const DIRECTORIES_LEN: usize = 0x200;
    pub static DIRECTORIES: Place<[u8; DIRECTORIES_LEN]> = Place::new([0; DIRECTORIES_LEN]);
    /// g_packedPath (0x5BE9BD .. 0x5BEAC0): the packed copy's path is built here.
    pub const PACKED_PATH_LEN: usize = 0x103;
    pub static PACKED_PATH: Place<[u8; PACKED_PATH_LEN]> = Place::new([0; PACKED_PATH_LEN]);
    /// byte_94E8B0 (the numbered paths, 0x100 bytes) and, after it, the resolved paths of
    /// a thread without a slot (0x94E9B0, 0x100 bytes).
    pub const PATHS_LEN: usize = 0x200;
    pub static PATHS: Place<[u8; PATHS_LEN]> = Place::new([0; PATHS_LEN]);

    /// g_baseDirectory.
    pub fn base_directory() -> *mut u8 {
        DIRECTORIES.ptr().cast()
    }

    /// g_localeDirectory.
    pub fn locale_directory() -> *mut u8 {
        unsafe { DIRECTORIES.ptr().cast::<u8>().add(0x100) }
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use crate::moved::image_bytes;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn state_starts_as_the_games() {
        // `files::search`'s test writes the search record.
        let _lock = crate::store::test_lock();
        let word = |ida| u32::from_le_bytes(image_bytes(ida, 4).try_into().unwrap());
        assert_eq!(word(0x4F52B0), 0, "no last path");
        assert_eq!(word(0x4E05A4), FIND_HANDLE.load(Relaxed));
        assert_eq!(word(0x4E06E6), SKIPPED_ATTRIBUTES.load(Relaxed));
        assert_eq!(word(0x60B2CC), BYTES_READ.load(Relaxed));
        assert_eq!(word(0x5189A4), MEDIA_MODE.load(Relaxed));
        let bytes = |p: *const u8, n| unsafe { std::slice::from_raw_parts(p, n).to_vec() };
        assert_eq!(
            bytes(FIND_DATA.ptr().cast(), 0x13E),
            image_bytes(0x4E05A8, 0x13E)
        );
        assert_eq!(
            bytes(PACKED_PATH.ptr().cast(), PACKED_PATH_LEN),
            image_bytes(0x5BE9BD, PACKED_PATH_LEN)
        );
        assert_eq!(
            bytes(DIRECTORIES.ptr().cast(), DIRECTORIES_LEN),
            image_bytes(0x5189A8, DIRECTORIES_LEN)
        );
        assert_eq!(
            bytes(PATHS.ptr().cast(), PATHS_LEN),
            image_bytes(0x94E8B0, PATHS_LEN)
        );
    }
}
