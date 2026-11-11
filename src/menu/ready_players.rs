//! `menu::ready_players` (0x471F7E): each counted player (0 .. `event.player_count`, the
//! original's `do .. while`) taking part again: +0xD0 1, +0x88 0, +0x94 0.
//!
//! Callers: `saves::load_savegame`, the single rally's and the time trial's multi-player events.
//! Refuses (before anything) a player count of 0 or past 4 or a player missing .

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::counted_players;

pub fn ready_players(menus: &Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::Player::ready`.
    for p in counted_players(menus)? {
        p.update(rc2k_core::menu::player::Player::ready);
    }
    Ok(())
}
