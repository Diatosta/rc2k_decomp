//! `menu::toggle_map_area` (0x467DE2): the player's visual preference "mapa local" the other way.
//!
//! `map_area` xor 1. The left and right callbacks of its item
//! on the visual preferences page (menu table 0x748280). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.map_area
}

// ---- Rust version ----

pub fn toggle_map_area() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::MapArea, value, true)
    })
}
