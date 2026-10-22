//! `menu::next_second_view` (0x467B8D): the player's next second view.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `second_view` + 1, back to 0 after 4. An action of the four players' details pages (menu tables 0x74EAEC, 0x74EEE8).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.second_view
}

// ---- Rust version ----

pub fn next_second_view() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::SecondView, value, true)
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_second_view()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
