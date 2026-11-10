//! `menu::reset_player_totals` (0x471FB8): the counted players' (`dword_755984`) +0x8C and
//! +0x88 3600000.
//!
//! Refuses a player count of 0 or past 4 or a missing player, before
//! anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn reset_player_totals(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::menu::player::Player::reset_totals`.
    for p in counted_players(screens)? {
        p.update(rc2k_core::menu::player::Player::reset_totals);
    }
    Ok(())
}
