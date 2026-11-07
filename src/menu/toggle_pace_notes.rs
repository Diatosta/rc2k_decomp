//! `menu::toggle_pace_notes` (0x467E7C): the player's visual preference "notas informativas" the other way.
//!
//! `pace_notes` xor 1. The left and right callbacks of its item
//! on the visual preferences page (menu table 0x748280). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.pace_notes
}

// ---- Rust version ----

pub fn toggle_pace_notes() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::PaceNotes, value, true)
    })
}
