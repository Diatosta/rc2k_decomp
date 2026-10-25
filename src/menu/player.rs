//! A player, 0x28C bytes (`Player` in IDA): the four at `g_player1` .. `g_player4`,
//! pointed at by `g_players` (`off_75647C`): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps. The service area's and
//! the saves' code still read it by offsets.

use crate::menu::players_store::PLAYER_TABLE;

pub use rc2k_core::menu::player::{PLAYERS, Player};

/// Player `index`'s address, or 0.
pub fn player_at(index: u32) -> u32 {
    if index >= PLAYERS {
        return 0;
    }
    PLAYER_TABLE.pointers[index as usize].get()
}
