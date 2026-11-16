//! Finds the highest number of the viewer's pictures (`savedata\snapshot\000.pcx`): the name's
//! digits are made `???` and searched for (`files::find_first`, `files::find_next_wanted`,
//! `files::close_search`, the path not resolved, `last_file_path` the pattern); of each name found
//! whose first three characters are digits the number is taken, and the highest (0 for none) is
//! written back over the `???`. `race::viewer_snapshot` counts on from it. Called by
//! `race::replay_viewer`.

use crate::errors::BadInput;
use crate::files::search::DIRECTORY;
use crate::files::state::set_last_file_path;
use crate::race::owner_replay::Replay;

/// The three digits' place in the name.
pub const DIGITS_AT: usize = 0x12;

/// `files::find_next_wanted`: the next name not of the skipped attributes; false when
/// there is none.
pub fn find_next_wanted(skipped: u32) -> bool {
    crate::files::search::next(skipped)
}

/// `files::find_first`: the first name matching `pattern` (without its NUL; not
/// resolved; it becomes `last_file_path`) not of the skipped attributes; false when none.
pub fn find_first(pattern: &[u8], skipped: u32) -> bool {
    set_last_file_path(Some(pattern));
    crate::files::search::first(pattern, skipped)
}

pub fn find_snapshot_number(replay: &Replay) -> Result<(), BadInput> {
    // The name's bytes: a file name.
    replay
        .snapshot()
        .write_at(DIGITS_AT, b"???.", "the snapshot name")?;
    let mut highest = 0u32;
    // The pattern: the snapshot name's text, read where it is named.
    let found = match crate::files::cstring(replay.snapshot(), "a search pattern") {
        Ok(pattern) => find_first(&pattern, DIRECTORY),
        Err(_) => {
            set_last_file_path(None);
            crate::files::search::close();
            false
        }
    };
    if found {
        loop {
            let name = crate::files::search::found_name();
            if let Some(number) =
                rc2k_core::race::viewer::snapshot_number(|n| name.get(n).copied().unwrap_or(0))
                && number > highest
            {
                highest = number;
            }
            if !find_next_wanted(DIRECTORY) {
                break;
            }
        }
    }
    crate::files::search::close();
    let digits = crate::strings::format_u32::format_u32(highest, 0x13);
    replay
        .snapshot()
        .write_at(DIGITS_AT, &digits, "the snapshot name")?;
    Ok(())
}
