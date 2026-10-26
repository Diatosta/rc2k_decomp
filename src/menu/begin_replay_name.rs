//! `menu::begin_replay_name` (0x46F715): starts typing the name of the highlighted replay:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `dword_7556A8` (an entry is going on) is 1.
//! It first logs "getrepsaveslotname result=<name>".
//!
//! In:  nothing
//! Out: every register preserved
//! It writes "Backed up name=<the name>" to the game's log (`logfile::log_write`). In six menu
//! tables. Refuses a name past the list. Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::menu::name_lists::replay_name;
use crate::menu::text_entry::{begin_logged, log_l1};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// `log` = false leaves the log lines out.
pub unsafe fn begin_replay_name(log: bool) -> Result<(), BadInput> {
    let name = replay_name()?;
    unsafe {
        if log {
            log_l1(game_addr(rva(0x4DE1F3)) as *const u8, false); // aGetrepsaveslot
            log_l1(name, true);
        }
        // Converted: `rc2k_core::menu::text_entry::Entry::begin` (through `begin_logged`).
        begin_logged(name, log);
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_replay_name(true) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
