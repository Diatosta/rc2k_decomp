//! `menu::flip_codriver` (0x467BD6): the player's co-driver's sex the other.
//!
//! In:  nothing
//! Out: edi = the player; everything else preserved
//!
//! `codriver_sex` xor 1. No `pusha`: edi is left the player. An action of the four players'
//! details pages (menu tables 0x74EBA4 ..). Hook removed 2026-10-22 (typed tables): the menus
//! and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::{EDI, Regs};
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.codriver_sex
}

// ---- Rust version ----

pub fn flip_codriver() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::CodriverSex, value, true)
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    flip_codriver()?;
    Ok(Left::nothing().with(EDI, current()? as u32))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
