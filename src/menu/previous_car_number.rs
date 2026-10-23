//! `menu::previous_car_number` (0x470DA1): the player's car number down by one.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `race_number` - 1, to 90 at 0 or below (signed). An action of the four players' details
//! pages (menu tables 0x74EB48 ..).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.race_number
}

// ---- Rust version ----

pub fn previous_car_number() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::RaceNumber, value, false)
    })
}
