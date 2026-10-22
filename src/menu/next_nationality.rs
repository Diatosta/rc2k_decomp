//! `menu::next_nationality` (0x467AFD): the player's next nationality.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `nationality` + 1, back to 0 after 0x27 (the 40 nationalities). An action of the four
//! players' details pages (menu tables 0x74EA34 ..). Hook removed 2026-10-22 (typed tables):
//! the menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.nationality
}

// ---- Rust version ----

pub fn next_nationality() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::Nationality, value, true)
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_nationality()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
