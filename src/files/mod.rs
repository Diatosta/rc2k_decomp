pub mod append_directory;
pub mod append_file_name;
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

/// The NUL-terminated path at `at` (the game's ANSI bytes) as a `PathBuf`.
pub unsafe fn ansi_path(at: *const u8) -> std::path::PathBuf {
    let bytes = unsafe { std::ffi::CStr::from_ptr(at as *const std::ffi::c_char) }.to_bytes();
    std::path::PathBuf::from(ansi_text(bytes))
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
    }
}

/// The file code's variables, out of the image since 2026-10-24 (phase 5; each has its own
/// entry in `moved::MOVED`).
pub mod state {
    use crate::moved::Place;
    use std::sync::atomic::AtomicU32;
    use windows_sys::Win32::Storage::FileSystem::WIN32_FIND_DATAA;

    /// g_lastFilePath (0x4F52B0): the path of the last file opened or searched for, or 0
    /// (the fatal-error report names it).
    pub static LAST_FILE_PATH: AtomicU32 = AtomicU32::new(0);
    /// hFindFile (0x4E05A4): the search handle, `u32::MAX` for none.
    pub static FIND_HANDLE: AtomicU32 = AtomicU32::new(u32::MAX);
    /// FindFileData (0x4E05A8, `WIN32_FIND_DATAA`, 0x13E bytes; 0x140 here): the search's
    /// record.
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

    /// The search record.
    pub fn find_data() -> *mut WIN32_FIND_DATAA {
        FIND_DATA.ptr().cast()
    }
}

#[cfg(test)]
mod tests {
    use super::state::*;
    use crate::moved::image_bytes;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn state_starts_as_the_games() {
        let word = |ida| u32::from_le_bytes(image_bytes(ida, 4).try_into().unwrap());
        assert_eq!(word(0x4F52B0), LAST_FILE_PATH.load(Relaxed));
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
