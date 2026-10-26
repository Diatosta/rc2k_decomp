//! A player, 0x28C bytes (`Player` in IDA): the four at `g_player1` .. `g_player4`,
//! pointed at by `g_players` (`off_75647C`): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps. The service area's and
//! the saves' code still read it by offsets.

use crate::framework::{game_read, rva};

pub const PLAYERS_RVA: usize = rva(0x75647C); // g_players

pub use rc2k_core::menu::player::{PLAYERS, Player, Setting};

/// Player `index`'s address, or 0.
pub fn player_at(index: u32) -> u32 {
    if index >= PLAYERS {
        return 0;
    }
    unsafe { game_read(PLAYERS_RVA + index as usize * 4) }
}

/// The four players, as harness regions (for self-tests and shadow snapshots).
pub fn player_regions() -> Vec<crate::service::harness::Region> {
    (0..PLAYERS)
        .map(player_at)
        .filter(|&p| p != 0)
        .map(|p| (p, std::mem::size_of::<Player>()))
        .collect()
}
