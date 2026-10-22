//! `service::next_setup_page` (0x46F191): shows the next page of the setup slot list: adds 1 to
//! `dword_755244`, and from 3 on goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//! The comparison is signed, so a negative page just goes up by one. In two menu tables.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::game_read;
use crate::framework::{Regs, game_write};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::service::load_setup_slot::SLOT_PAGE_RVA;

// ---- Rust version ----

pub fn next_setup_page() {
    unsafe {
        // Converted: `rc2k_core::service::setups::next_page`.
        game_write(
            SLOT_PAGE_RVA,
            rc2k_core::service::setups::next_page(game_read(SLOT_PAGE_RVA)),
        );
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_setup_page();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
