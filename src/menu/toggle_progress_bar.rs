//! `menu::toggle_progress_bar` (0x467E50): the player's visual preference "barra de progresso" the other way.
//!
//! In:  nothing
//! Out: edi = the player; everything else preserved
//!
//! `progress_bar` xor 1. No `pusha`: edi is left the player. The left and right callbacks of its
//! item on the visual preferences page (menu table 0x748280). The menus and draw lists run its
//! `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

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
