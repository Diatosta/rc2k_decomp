//! `race::find_snapshot_number` (0x41EF51): the viewer's pictures' name
//! (`savedata\snapshot\000.pcx`) given the highest number of the pictures there: its digits made
//! `???` and searched for (`files::find_first`, `files::find_next_wanted`, `files::close_search`:
//! `FindFirstFileA` and on in the game's search record `FindFileData` and handle `hFindFile`, the
//! path not resolved, `g_lastFilePath` the pattern); of each name found whose first three
//! characters are digits the number, the highest (0 for none) written back over the `???`.
//! `race::viewer_snapshot` counts on from it.
//!
//! Caller: `race::replay_viewer`.

use crate::errors::BadInput;
use crate::files::state::{SKIPPED_ATTRIBUTES, set_last_file_path};
#[cfg(test)]
#[cfg(test)]
use crate::race::owner::race;
use crate::race::owner_replay::Replay;

/// IDA's address of the snapshot picture's name.
pub const SNAPSHOT_START: usize = 0x529731;
/// The three digits' place in the name.
pub const DIGITS_AT: usize = 0x12;

/// `files::find_next_wanted` (0x402FF6): the next name not of the skipped attributes; false when
/// there is none.
pub fn find_next_wanted() -> bool {
    crate::files::search::next(SKIPPED_ATTRIBUTES.get())
}

/// `files::find_first` (0x402FC2): the first name matching `pattern` (without its NUL; not
/// resolved; it becomes `g_lastFilePath`) not of the skipped attributes; false when none.
pub fn find_first(pattern: &[u8]) -> bool {
    set_last_file_path(Some(pattern));
    crate::files::search::first(pattern, SKIPPED_ATTRIBUTES.get())
}

pub fn find_snapshot_number(replay: &Replay) -> Result<(), BadInput> {
    // The name's bytes: a file name.
    replay
        .snapshot
        .write_at(DIGITS_AT, b"???.", "the snapshot name")?;
    let mut highest = 0u32;
    // The pattern: the snapshot name's text, read where it is named.
    let found = match crate::files::cstring(&replay.snapshot, "a search pattern") {
        Ok(pattern) => find_first(&pattern),
        Err(_) => {
            set_last_file_path(None);
            crate::files::search::close();
            false
        }
    };
    if found {
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
    replay
        .snapshot
        .write_at(DIGITS_AT, &digits, "the snapshot name")?;
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
