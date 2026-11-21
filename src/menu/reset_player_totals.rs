//! `menu::reset_player_totals`: the counted players' (`event.player_count`) `total` and
//! `time` set to 3600000.
//!
//! Refuses a player count of 0 or past 4 or a missing player, before
//! anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn reset_player_totals(menus: &Menus) -> Result<(), Refused> {
    for p in counted_players(&menus.screens, &menus.play)? {
        p.update(rc2k_core::menu::player::Player::reset_totals);
    }
    Ok(())
}
