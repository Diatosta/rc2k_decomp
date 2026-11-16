//! `menu::ready_players`: each counted player (0 .. `event.player_count`) taking part again:
//! +0xD0 1, +0x88 0, +0x94 0.
//!
//! Refuses (before anything) a player count of 0 or past 4 or a player missing.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::counted_players;

pub fn ready_players(menus: &Menus) -> Result<(), Refused> {
    for p in counted_players(&menus.screens, &menus.play)? {
        p.update(rc2k_core::menu::player::Player::ready);
    }
    Ok(())
}
