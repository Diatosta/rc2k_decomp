//! `menu::clear_hud_bits`: every player's `hud_bits` set to -1.
//!
//! All four players. Called at the start of each kind of event (`menu::single_rally`
//! and seven more).

use crate::errors::Refused;
use crate::menu::owner::Menus;

pub fn clear_hud_bits(menus: &Menus) -> Result<(), Refused> {
    for player in &menus.play.players.players {
        player.update(|p| p.hud_bits = -1);
    }
    Ok(())
}
