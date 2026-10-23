//! `menu::raise_sensitivity` (0x470F05): the player's controller sensitivity up by one.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `controller_sensitivity` + 1, held at 100 (signed). An action of the four players' details
//! pages (menu tables 0x74EC00 ..). Hook removed 2026-10-22 (typed tables): the menus and draw
//! lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.controller_sensitivity
}

// ---- Rust version ----

pub fn raise_sensitivity() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::Sensitivity, value, true)
    })
}
