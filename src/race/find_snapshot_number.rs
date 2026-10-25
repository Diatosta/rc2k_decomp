//! `race::find_snapshot_number` (0x41EF51): the viewer's pictures' name
//! (`savedata\snapshot\000.pcx`) given the highest number of the pictures there: its
//! digits made `???` and searched for (`files::find_first`, `files::find_next_wanted`,
//! `files::close_search`: `FindFirstFileA` and on in the game's search record
//! `FindFileData` and handle `hFindFile`, the path not resolved, `g_lastFilePath` the
//! pattern); of each name found whose first three characters are digits the number, the
//! highest (0 for none) written back over the `???` (`strings::format_u32`, flags 0x13).
//! `race::viewer_snapshot` counts on from it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::replay_viewer`.

use crate::files::state::{FIND_HANDLE, LAST_FILE_PATH, SKIPPED_ATTRIBUTES, find_data};
use crate::session::Bytes;
use std::sync::atomic::Ordering::Relaxed;
use windows_sys::Win32::Storage::FileSystem::{FindClose, FindFirstFileA, FindNextFileA};

/// IDA's address of the snapshot picture's name.
pub const SNAPSHOT_START: usize = 0x529731;
/// The three digits' place in the name.
pub const DIGITS_AT: usize = 0x12;

/// `aSavedataSnapsh`: the viewer's snapshot picture's name, its three digits counted up.
pub static SNAPSHOT: Bytes<26> = Bytes::new(*b"savedata\\snapshot\\000.pcx\0");

/// Where the name is (`moved::MOVED`'s home for IDA 0x529731).
pub fn snapshot_home() -> usize {
    SNAPSHOT.addr()
}

/// Where the name's three digits are.
pub fn digits_place() -> *mut u8 {
    SNAPSHOT.as_ptr().wrapping_add(DIGITS_AT)
}

const NO_SEARCH: u32 = u32::MAX;

/// `files::find_next_wanted` (0x402FF6): the next name not of the skipped attributes;
/// false (carry) when there is none.
pub unsafe fn find_next_wanted() -> bool {
    let record = find_data();
    loop {
        if unsafe { FindNextFileA(FIND_HANDLE.load(Relaxed) as _, record) } == 0 {
            return false;
        }
        if unsafe { (*record).dwFileAttributes } & SKIPPED_ATTRIBUTES.load(Relaxed) == 0 {
            return true;
        }
    }
}

/// `files::find_first` (0x402FC2): the first name matching `pattern` (not resolved; it
/// becomes `g_lastFilePath`) not of the skipped attributes; false (carry) when none.
pub unsafe fn find_first(pattern: u32) -> bool {
    LAST_FILE_PATH.store(pattern, Relaxed);
    let record = find_data();
    let search = unsafe { FindFirstFileA(pattern as *const u8, record) } as u32;
    FIND_HANDLE.store(search, Relaxed);
    if search == NO_SEARCH {
        return false;
    }
    if unsafe { (*record).dwFileAttributes } & SKIPPED_ATTRIBUTES.load(Relaxed) != 0 {
        return unsafe { find_next_wanted() };
    }
    true
}

/// `files::close_search` (0x40301F): the search closed, if there is one.
pub unsafe fn close_search() {
    if FIND_HANDLE.load(Relaxed) != NO_SEARCH {
        unsafe { FindClose(FIND_HANDLE.load(Relaxed) as _) };
        FIND_HANDLE.store(NO_SEARCH, Relaxed);
    }
}

/// The Rust version.
pub unsafe fn find_snapshot_number() {
    unsafe {
        (digits_place() as *mut u32).write_unaligned(u32::from_le_bytes(*b"???."));
        let mut highest = 0u32;
        if find_first(SNAPSHOT.addr() as u32) {
            loop {
                // Converted: `rc2k_core::race::viewer::snapshot_number`.
                let name = &(*find_data()).cFileName;
                if let Some(number) = rc2k_core::race::viewer::snapshot_number(|n| name[n] as u8)
                    && number > highest
                {
                    highest = number;
                }
                if !find_next_wanted() {
                    break;
                }
            }
        }
        close_search();
        let digits = crate::strings::format_u32::format_u32(highest, 0x13);
        std::ptr::copy_nonoverlapping(digits.as_ptr(), digits_place(), digits.len());
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        crate::gfx::first_values_are_the_games(&SNAPSHOT, SNAPSHOT_START, SNAPSHOT_START + 26);
        assert_eq!(crate::moved::place(SNAPSHOT_START), SNAPSHOT.addr());
    }
}
