//! `menu::toggle_progress_bar` (0x467E50): the player's visual preference "barra de progresso" the other way.
//!
//! In:  nothing
//! Out: edi = the player; everything else preserved
//!
//! `progress_bar` xor 1. No `pusha`: edi is left the player. The left and right callbacks of its
//! item on the visual preferences page (menu table 0x748280).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{EDI, Regs};
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.progress_bar
}

// ---- Rust version ----

pub fn toggle_progress_bar() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::ProgressBar, value, true)
    })
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    toggle_progress_bar()?;
    Ok(Left::nothing().with(EDI, current()? as u32))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
