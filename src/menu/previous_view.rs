//! `menu::previous_view` (0x467B69): the player's view before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `view` - 1, 6 from 0. An action of the four players' details pages (menu tables 0x74EA90 ..).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.view
}

// ---- Rust version ----

pub fn previous_view() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::View, value, false)
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    previous_view()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
