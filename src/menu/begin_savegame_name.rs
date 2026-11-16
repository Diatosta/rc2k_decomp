//! `menu::begin_savegame_name`: starts typing the name of the highlighted saved game:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `entry.typing` (an entry is going on) is 1.
//!
//! It writes `Backed up name=<the name>` to the game's log (`logfile::log_write`). In six menu
//! tables. Refuses a name past the list.

use crate::errors::BadInput;
use crate::menu::name_lists::savegame_name;
use crate::menu::owner::Menus;
use crate::menu::text_entry::begin_logged;
use crate::saves::owner::Saves;

/// `log` = false leaves the log lines out.
pub fn begin_savegame_name(
    menus: &'static Menus,
    saves: &'static Saves,
    log: bool,
) -> Result<(), BadInput> {
    let name = savegame_name(menus, saves)?;
    begin_logged(menus, name, log)
}
