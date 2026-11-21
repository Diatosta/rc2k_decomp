//! `menu::begin_replay_name`: starts typing the name of the highlighted replay:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `entry.typing` (an entry is going on) is 1.
//! It first logs `getrepsaveslotname result=<name>`.
//!
//! It writes `Backed up name=<the name>` to the game's log (`logfile::log_write`). In six menu
//! tables. Refuses a name past the list.

use crate::errors::BadInput;
use crate::menu::name_lists::replay_name;
use crate::menu::owner::Menus;
use crate::menu::text_entry::begin_logged;
use crate::race::owner::Race;

/// `log` = false leaves the log lines out.
pub(super) fn begin_replay_name(
    menus: &'static Menus,
    race: &'static Race,
    log: bool,
) -> Result<(), BadInput> {
    let name = replay_name(menus, race)?;
    if log {
        // aGetrepsaveslot
        crate::logfile::log_write::log_nul(c"getrepsaveslotname result=", false, 1);
        crate::logfile::log_write::log_at(name, true, 1);
    }
    begin_logged(menus, name, log)
}
