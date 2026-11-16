//! `menu::add_player_times`: each counted player's (`event.player_count`) stage time
//! (+0x88) added to the total (+0x8C) when it took part (+0xD0 1), else both 3600000.
//!
//! Refuses a player count of 0 or past 4 or a missing player, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn add_player_times(menus: &Menus) -> Result<(), Refused> {
    for p in counted_players(&menus.screens, &menus.play)? {
        p.update(rc2k_core::menu::player::Player::add_time);
    }
    Ok(())
}
