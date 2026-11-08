//! A player, 0x28C bytes (`Player` in IDA): the four at `g_player1` .. `g_player4`,
//! pointed at by `g_players` (`off_75647C`): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps. The service area's and
//! the saves' code still read it by offsets.

use crate::menu::owner::menus;

pub use rc2k_core::menu::player::{PLAYERS, Player};

/// Player `index`, none past the four.
pub fn player(index: u32) -> Option<&'static crate::session::Shared<Player>> {
    menus().play.players.players.get(index as usize)
}
