//! `menu::next_savegame_page` (0x46D407): the next page of the saved games' list: adds
//! 1 to `dword_75523C`, and above 9 goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The comparison is signed. In two menu tables (0x7518D0, 0x751D28): the load and
//! save screens' page buttons. The twin of `service::next_setup_page`.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub fn next_savegame_page() {
    unsafe {
        // Converted: `rc2k_core::menu::steps::savegame_page`.
        let page = game_read::<u32>(SAVEGAME_PAGE_RVA);
        game_write(
            SAVEGAME_PAGE_RVA,
            rc2k_core::menu::steps::savegame_page(page, true),
        );
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_savegame_page();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
