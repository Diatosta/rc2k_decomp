//! `menu::take_player1_units`: the game's units from player 1's.
//!
//! The game's units setting (`strings::units::set_units`) = player 1's `metric_units`:
//! 1 for metric. Called at the start of each kind of event (`menu::single_rally` and more).

use crate::errors::Refused;
use crate::menu::owner::Menus;

pub fn take_player1_units(
    menus: &Menus,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    crate::strings::units::set_units(strings, menus.play.slots.records[0].get().metric_units);
    Ok(())
}
