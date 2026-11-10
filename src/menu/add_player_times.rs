//! `menu::add_player_times` (0x4A25AD): each counted player's (`dword_755984`) stage time
//! (+0x88) added to the total (+0x8C) when it took part (+0xD0 1), else both 3600000.
//!
//! Callers: the event drivers. Refuses a player count of 0 or past 4 or a missing player, before
//! anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn add_player_times(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::menu::player::Player::add_time`.
    for p in counted_players(screens)? {
        p.update(rc2k_core::menu::player::Player::add_time);
    }
    Ok(())
}
