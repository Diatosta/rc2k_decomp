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
use crate::files::state::{SKIPPED_ATTRIBUTES, set_last_file_path};
use crate::pointers::GamePtr;
#[cfg(test)]
#[cfg(test)]
use crate::race::owner::race;
use crate::race::owner_replay::Replay;

/// IDA's address of the snapshot picture's name.
pub const SNAPSHOT_START: usize = 0x529731;
/// The three digits' place in the name.
pub const DIGITS_AT: usize = 0x12;

/// Where the name's three digits are.
pub fn digits_place(replay: &Replay) -> GamePtr {
    // The name's bytes: a file name.
    GamePtr::to(&replay.snapshot).offset(DIGITS_AT as u32)
}

/// `files::find_next_wanted` (0x402FF6): the next name not of the skipped attributes;
/// false (carry) when there is none.
pub fn find_next_wanted() -> bool {
    crate::files::search::next(SKIPPED_ATTRIBUTES.get())
}

/// `files::find_first` (0x402FC2): the first name matching `pattern` (not resolved; it
/// becomes `g_lastFilePath`) not of the skipped attributes; false (carry) when none.
pub fn find_first(pattern: GamePtr) -> bool {
    find_first_at(pattern)
}

/// `find_first` for a pattern held in Rust's game state.
pub fn find_first_at(pattern: GamePtr) -> bool {
    let Ok(text) = crate::files::cstring(pattern, "a search pattern") else {
        set_last_file_path(None);
        crate::files::search::close();
        return false;
    };
    set_last_file_path(Some(&text));
    crate::files::search::first(&text, SKIPPED_ATTRIBUTES.get())
}

/// The Rust version.
pub fn find_snapshot_number(replay: &Replay) -> Result<(), BadInput> {
    // The name's bytes: a file name.
    GamePtr::to(&replay.snapshot)
        .offset(DIGITS_AT as u32)
        .write(u32::from_le_bytes(*b"???."), "the snapshot name")?;
    let mut highest = 0u32;
    if find_first_at(GamePtr::to(&replay.snapshot)) {
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
    GamePtr::to(&replay.snapshot)
        .offset(DIGITS_AT as u32)
        .write_bytes(&digits, "the snapshot name")?;
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        let replay = &race().replay;
        crate::gfx::first_values_are_the_games(
            &replay.snapshot,
            SNAPSHOT_START,
            SNAPSHOT_START + 26,
        );
        assert_eq!(
            crate::moved::place(SNAPSHOT_START),
            replay.snapshot.addr() as usize
        );
    }
}
