//! `menu::ask_to_leave` (0x47E8BB): `dword_755569` 1: the screen loop asks whether to leave
//! (the review menus' "back to the main menu", a reading).
//!
//! In:  nothing
//! Out: every register kept
//!
//! An item action in six menu tables (the review menus).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header).
pub fn ask_to_leave() {
    // Converted: `rc2k_core::menu::leaving::Leaving::ask_to_leave`.
    crate::menu::leaving::with_leaving(|l| l.ask_to_leave());
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    ask_to_leave();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
