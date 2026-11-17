//! Reading and writing files. The game names its files as a 1999 Windows program does, with
//! backslashes and without regard to case; this module turns such a name into a real path on the
//! system it runs on (`resolve_path`) and offers the ways the game reads a file: whole, packed
//! (`read_packed_file`), or through a small buffered stream.

pub mod close_stream;
pub mod copy_file;
pub mod directories;
pub mod drive_free_space_kb;
pub mod drive_type;
pub mod file_exists;
pub mod file_size;
pub mod handle;
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

pub use rc2k_core::files::ansi::{ansi_bytes, ansi_text};

/// The text in `bytes` up to its NUL (not included); a text with no NUL in them is refused.
pub fn cstring_in(bytes: &[u8]) -> Result<Vec<u8>, crate::errors::BadInput> {
    match bytes.iter().position(|&b| b == 0) {
        Some(end) => Ok(bytes[..end].to_vec()),
        None => Err(crate::errors::BadInput(
            "a text with no end in its allocation",
        )),
    }
}

/// The text at byte `offset` of the store, read within its next `len` bytes.
pub fn cstring_from(
    store: &dyn crate::memory::cells::Store,
    offset: usize,
    len: usize,
) -> Result<Vec<u8>, crate::errors::BadInput> {
    let mut bytes = vec![0u8; len];
    store.load(offset, &mut bytes)?;
    cstring_in(&bytes)
}

pub fn cstring(
    at: &dyn crate::memory::cells::Store,
    _what: &str,
) -> Result<Vec<u8>, crate::errors::BadInput> {
    cstring_from(at, 0, at.size())
}

/// A path in the game's bytes (no NUL) as a `PathBuf`: the one place a game path becomes the
/// system's. A numbered instance (`--instance`) finds its `var` and `savedata` folders under
/// `instances/<n>/` (`single_copy::instance_path`).
pub fn host_path(path: &[u8]) -> std::path::PathBuf {
    match crate::single_copy::instance_path(path) {
        Some(own) => host_path_of(&own),
        None => host_path_of(path),
    }
}

/// `host_path` for a path the instance does not redirect (the folder's own).
pub fn shared_host_path(path: &[u8]) -> std::path::PathBuf {
    host_path_of(path)
}

/// A path in the game's bytes as the system's, as read on Windows.
#[cfg(windows)]
fn host_path_of(path: &[u8]) -> std::path::PathBuf {
    std::path::PathBuf::from(ansi_text(path))
}

/// A path in the game's bytes (no NUL) as a `PathBuf`: the one place a game path becomes
/// the system's. On Unix as Windows read it: `\` a separator as `/` is, a drive
/// letter (`c:`) the root, and each name found in its folder without regard to case (the
/// game's files are named in any case); a name that is not there (a file to be made) is
/// kept as written.
#[cfg(unix)]
fn host_path_of(path: &[u8]) -> std::path::PathBuf {
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

#[cfg(test)]
mod ansi_tests {
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

pub mod state {

    /// The path of the last file opened or searched for, or none (the fatal-error report names
    /// it).
    static LAST_FILE_PATH: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);

    /// Set to `path` (without its NUL), or to none.
    pub fn set_last_file_path(path: Option<&[u8]>) {
        *LAST_FILE_PATH
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = path.map(<[u8]>::to_vec);
    }

    pub fn last_file_path() -> Option<Vec<u8>> {
        LAST_FILE_PATH
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    /// How the game is installed (0 .. 3), as the installer's record says; `resolve_path` looks at
    /// it.
    pub static MEDIA_MODE: crate::memory::session::Dword = crate::memory::session::Dword::new(0);
    /// The base directory (0x100 bytes) and, after it, the locale directory (0x100 bytes):
    /// strings, empty at the start.
    pub const DIRECTORIES_LEN: usize = 0x200;
    pub static DIRECTORIES: crate::memory::session::Bytes<DIRECTORIES_LEN> =
        crate::memory::session::Bytes::new([0; DIRECTORIES_LEN]);
    /// The second base directory: an empty text, never written (the installer's calls borrow it
    /// for as long as they run).
    pub static SECOND_BASE: crate::memory::session::Bytes<1> =
        crate::memory::session::Bytes::new(*b"\0");
    /// The room of the packed copy's path.
    pub const PACKED_PATH_LEN: usize = 0x103;
    /// The numbered paths (0x100 bytes) and, after them, the resolved paths of a thread without a
    /// slot (0x100 bytes).
    pub const PATHS_LEN: usize = 0x200;
    pub static PATHS: crate::memory::session::Bytes<PATHS_LEN> =
        crate::memory::session::Bytes::new([0; PATHS_LEN]);

    /// A directory's room (`DIRECTORIES` holds the two).
    pub const DIRECTORY_LEN: u32 = 0x100;

    /// Directory `n`'s first byte (0 base, 1 locale).
    pub fn directory_byte(n: usize) -> u8 {
        DIRECTORIES.get(n * DIRECTORY_LEN as usize)
    }

    pub fn set_directory_byte(n: usize, byte: u8) {
        DIRECTORIES.set(n * DIRECTORY_LEN as usize, byte);
    }

    /// Writes `text` from the start of directory `n` (and a NUL after it when `nul`); a text
    /// that does not fit in the room is refused.
    pub fn write_directory(
        n: usize,
        text: &[u8],
        nul: bool,
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        let room = DIRECTORY_LEN as usize;
        if text.len() + usize::from(nul) > room {
            return Err(crate::errors::BadInput("a directory text past its room"));
        }
        DIRECTORIES.write_at(n * room, text, what)?;
        if nul {
            DIRECTORIES.write_at(n * room + text.len(), &[0], what)?;
        }
        Ok(())
    }

    /// Directory `n`'s text (0 base, 1 locale), up to its NUL (a text with no NUL in its room is
    /// refused).
    pub fn directory_text(n: usize, what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        let room = DIRECTORY_LEN as usize;
        let bytes = DIRECTORIES.read_at(n * room, room, what)?;
        let end = bytes
            .iter()
            .position(|&b| b == 0)
            .ok_or(crate::errors::BadInput(
                "a directory without its end in its room",
            ))?;
        Ok(bytes[..end].to_vec())
    }
}
