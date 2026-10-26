//! The four players' names and the players that keep a copy of them.

use crate::errors::BadInput;
use crate::menu::names_store::NAME_BUFFERS;
use crate::menu::players_store::PLAYER_TABLE;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;
/// In a player: the address of its player's name, and a copy of the name.
pub const CAR_NAME_POINTER: usize = 0x18;
pub const CAR_NAME: usize = 0x124;

/// Where player `player`'s name is (the address the cars keep).
pub fn name(player: usize) -> u32 {
    (NAME_BUFFERS.player_names.addr() + player * NAME_LEN) as u32
}

/// Player `player`'s name, the whole 0x20 bytes.
pub fn name_bytes(player: usize) -> [u8; NAME_LEN] {
    std::array::from_fn(|i| NAME_BUFFERS.player_names.get(player * NAME_LEN + i))
}

/// The four players' addresses. Refuses a missing one.
pub fn cars() -> Result<[u32; PLAYERS], BadInput> {
    let mut cars = [0u32; PLAYERS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = PLAYER_TABLE.pointers[n].get();
        if *car == 0 {
            return Err(BadInput("a player is missing"));
        }
    }
    Ok(cars)
}
