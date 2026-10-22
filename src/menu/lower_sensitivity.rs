//! `menu::lower_sensitivity` (0x470F2D): the player's controller sensitivity down by one.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `controller_sensitivity` - 1, held at 0 (signed). An action of the four players' details
//! pages (menu tables 0x74EC00 ..). Hook removed 2026-10-22 (typed tables): the menus and draw
//! lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.controller_sensitivity
}

// ---- Rust version ----

pub fn lower_sensitivity() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(
            rc2k_core::menu::player::Setting::Sensitivity,
            value,
            false,
        )
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    lower_sensitivity()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
