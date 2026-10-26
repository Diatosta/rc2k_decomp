//! `menu::previous_view` (0x467B69): the player's view before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `view` - 1, 6 from 0. An action of the four players' details pages (menu tables 0x74EA90 ..).
//! The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.view
}

// ---- Rust version ----

pub fn previous_view() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::View, value, false)
    })
}
