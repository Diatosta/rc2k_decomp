//! `menu::flip_codriver`: the player's co-driver's sex the other.
//!
//! `codriver_sex` xor 1. An action of the four players' details pages.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::Player;
use crate::menu::player_settings::*;

fn field(player: &mut Player) -> &mut u32 {
    &mut player.codriver_sex
}

pub(super) fn flip_codriver(menus: &Menus) -> Result<(), Refused> {
    change(menus, field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::CodriverSex, value, true)
    })
}
