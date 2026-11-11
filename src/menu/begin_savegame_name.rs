//! `menu::begin_savegame_name` (0x46D571): starts typing the name of the highlighted saved game:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `option_work.typing` (an entry is going on) is 1.
//!
//! It writes "Backed up name=<the name>" to the game's log (`logfile::log_write`). In six menu
//! tables. Refuses a name past the list. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::BadInput;
use crate::menu::name_lists::savegame_name;
use crate::menu::owner::Menus;
use crate::menu::text_entry::begin_logged;

/// `log` = false leaves the log lines out.
pub fn begin_savegame_name(menus: &Menus, log: bool) -> Result<(), BadInput> {
    let name = savegame_name()?;
    // The logic is in `rc2k_core::menu::text_entry::Entry::begin` (through `begin_logged`).
    begin_logged(menus, name, log)
}
