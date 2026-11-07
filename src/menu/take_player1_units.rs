//! `menu::take_player1_units` (0x467EA8): the game's units from player 1's.
//!
//! `g_isImperialUnits` = player 1's `metric_units` (+0x178; despite the name the variable holds 1
//! for metric). Called at the start of each kind of event (`menu::single_rally` and more).

use crate::errors::Refused;
use crate::menu::owner::Menus;

pub fn take_player1_units(menus: &Menus) -> Result<(), Refused> {
    // g_isImperialUnits
    crate::strings::units::set_units(menus.play.players.players[0].get().metric_units);
    Ok(())
}
