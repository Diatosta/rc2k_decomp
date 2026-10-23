//! `menu::toggle_rev_counter` (0x467E0E): the player's visual preference "contador de rotações" the other way.
//!
//! In:  nothing
//! Out: edi = the player; everything else preserved
//!
//! `rev_counter` xor 1. No `pusha`: edi is left the player. The left and right callbacks of its
//! item on the visual preferences page (menu table 0x748280).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.rev_counter
}

// ---- Rust version ----

pub fn toggle_rev_counter() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::RevCounter, value, true)
    })
}
