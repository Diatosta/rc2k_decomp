//! The four players' names and the players that keep a copy of them.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, rva};
use crate::service::state::PLAYERS_RVA;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;
/// g_playerNames: four names of 0x20 bytes.
pub const NAMES_RVA: usize = rva(0x755784);
/// In a player: the address of its player's name, and a copy of the name.
pub const CAR_NAME_POINTER: usize = 0x18;
pub const CAR_NAME: usize = 0x124;

pub fn name(player: usize) -> *mut u8 {
    game_addr(NAMES_RVA + player * NAME_LEN) as *mut u8
}

/// The four players' addresses. Refuses a missing one.
pub unsafe fn cars() -> Result<[u32; PLAYERS], BadInput> {
    let mut cars = [0u32; PLAYERS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = unsafe { game_read(PLAYERS_RVA + n * 4) };
        if *car == 0 {
            return Err(BadInput("a player is missing"));
        }
    }
    Ok(cars)
}
