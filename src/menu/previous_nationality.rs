//! `menu::previous_nationality` (0x467B21): the player's nationality before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `nationality` - 1, 0x27 from 0. An action of the four players' details pages (menu tables
//! 0x74EA34 ..). The menus and draw lists run its `entry` (`menu::table_calls`); the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.nationality
}

// ---- Rust version ----

pub fn previous_nationality() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(
            rc2k_core::menu::player::Setting::Nationality,
            value,
            false,
        )
    })
}
