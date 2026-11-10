//! `menu::toggle_metric_units` (0x467E92): the player's visual preference "indicador de unidades" the other way.
//!
//! `metric_units` xor 1. The left and right callbacks of its
//! item on the visual preferences page (menu table 0x748280). The menus and draw lists run its
//! `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.metric_units
}

pub fn toggle_metric_units() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::MetricUnits, value, true)
    })
}
