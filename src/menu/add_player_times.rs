//! `menu::add_player_times`: each counted player's (`event.player_count`) stage time
//! (`time`) added to the event total (`total`) when the player took part (`entry` is
//! `TAKES_PART`), else both are set to 3600000 (an hour: no time).
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
