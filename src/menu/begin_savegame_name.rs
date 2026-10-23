//! `menu::begin_savegame_name` (0x46D571): starts typing the name of the highlighted saved game:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `dword_7556A8` (an entry is going on) is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! It writes "Backed up name=<the name>" to the game's log (`logfile::log_write`). In six menu
//! tables. Refuses a name past the list. Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::BadInput;
use crate::menu::name_lists::savegame_name;
use crate::menu::text_entry::begin_logged;

// ---- Rust version ----

/// `log` = false leaves the log lines out.
pub unsafe fn begin_savegame_name(log: bool) -> Result<(), BadInput> {
    let name = savegame_name()?;
    // Converted: `rc2k_core::menu::text_entry::Entry::begin` (through `begin_logged`).
    unsafe {
        begin_logged(name, log);
    }
    Ok(())
}
