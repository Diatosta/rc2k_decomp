//! A player, 0x28C bytes: the four player records, pointed at by `player_table.pointers`: the
//! player's settings (name, nationality, views, co-driver, keys, controller) and the state of the
//! player's car (which car, damage, setup, wear). What is known of it; the rest are gaps. The
//! service area's and the saves' code still read it by offsets.

use crate::menu::owner::Menus;

pub use rc2k_core::menu::player::{PLAYERS, Player};

/// Player `index`, none past the four.
pub fn player(menus: &Menus, index: u32) -> Option<&crate::memory::session::Shared<Player>> {
    menus.play.slots.records.get(index as usize)
}
