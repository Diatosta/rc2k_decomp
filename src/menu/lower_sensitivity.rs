//! `menu::lower_sensitivity` (0x470F2D): the player's controller sensitivity down by one.
//!
//! `controller_sensitivity` - 1, held at 0 (signed). An action of the four players' details pages
//! (menu tables 0x74EC00 ..). The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.controller_sensitivity
}

pub fn lower_sensitivity(menus: &Menus) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::player::stepped`.
    change(menus, field, |value| {
        rc2k_core::menu::player::stepped(
            rc2k_core::menu::player::Setting::Sensitivity,
            value,
            false,
        )
    })
}
