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

use crate::errors::BadInput;
use crate::files::state::{LAST_FILE_PATH, SKIPPED_ATTRIBUTES};
use crate::pointers::GamePtr;
use crate::session::Bytes;
use std::sync::atomic::Ordering::Relaxed;

/// IDA's address of the snapshot picture's name.
pub const SNAPSHOT_START: usize = 0x529731;
/// The three digits' place in the name.
pub const DIGITS_AT: usize = 0x12;

/// `aSavedataSnapsh`: the viewer's snapshot picture's name, its three digits counted up.
pub static SNAPSHOT: Bytes<26> = Bytes::new(*b"savedata\\snapshot\\000.pcx\0");

/// Where the name is (`moved::MOVED`'s home for IDA 0x529731).
pub fn snapshot_home() -> usize {
    SNAPSHOT.addr() as usize
}

/// Where the name's three digits are.
pub fn digits_place() -> *mut u8 {
    SNAPSHOT.as_ptr().wrapping_add(DIGITS_AT)
}

/// `files::find_next_wanted` (0x402FF6): the next name not of the skipped attributes;
/// false (carry) when there is none.
pub fn find_next_wanted() -> bool {
    crate::files::search::next(SKIPPED_ATTRIBUTES.load(Relaxed))
}

/// `files::find_first` (0x402FC2): the first name matching `pattern` (not resolved; it
/// becomes `g_lastFilePath`) not of the skipped attributes; false (carry) when none.
pub fn find_first(pattern: u32) -> bool {
    LAST_FILE_PATH.store(pattern, Relaxed);
    let Ok(text) = crate::files::cstring(GamePtr::new(pattern), "a search pattern") else {
        crate::files::search::close();
        return false;
    };
    crate::files::search::first(&text, SKIPPED_ATTRIBUTES.load(Relaxed))
}

/// The Rust version.
pub fn find_snapshot_number() -> Result<(), BadInput> {
    GamePtr::new(crate::pointers::game_address(digits_place()))
        .write(u32::from_le_bytes(*b"???."), "the snapshot name")?;
    let mut highest = 0u32;
    if find_first(SNAPSHOT.addr()) {
        loop {
            // Converted: `rc2k_core::race::viewer::snapshot_number`.
            let name = crate::files::search::found_name();
            if let Some(number) =
                rc2k_core::race::viewer::snapshot_number(|n| name.get(n).copied().unwrap_or(0))
                && number > highest
            {
                highest = number;
            }
            if !find_next_wanted() {
                break;
            }
        }
    }
    crate::files::search::close();
    let digits = crate::strings::format_u32::format_u32(highest, 0x13);
    GamePtr::new(crate::pointers::game_address(digits_place()))
        .write_bytes(&digits, "the snapshot name")?;
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        crate::gfx::first_values_are_the_games(&SNAPSHOT, SNAPSHOT_START, SNAPSHOT_START + 26);
        assert_eq!(
            crate::moved::place(SNAPSHOT_START),
            SNAPSHOT.addr() as usize
        );
    }
}
