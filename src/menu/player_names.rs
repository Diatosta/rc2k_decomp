//! The four players' names and the car objects that keep a copy of them.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, rva};
use crate::service::harness::Region;
use crate::service::state::CARS_RVA;

pub const PLAYERS: usize = 4;
pub const NAME_LEN: usize = 0x20;
/// g_playerNames: four names of 0x20 bytes.
pub const NAMES_RVA: usize = rva(0x755784);
/// In a car object: the address of its player's name, and a copy of the name.
pub const CAR_NAME_POINTER: usize = 0x18;
pub const CAR_NAME: usize = 0x124;

pub fn name(player: usize) -> *mut u8 {
    game_addr(NAMES_RVA + player * NAME_LEN) as *mut u8
}

/// The four car objects' addresses. Refuses a missing one.
pub unsafe fn cars() -> Result<[u32; PLAYERS], BadInput> {
    let mut cars = [0u32; PLAYERS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = unsafe { game_read(CARS_RVA + n * 4) };
        if *car == 0 {
            return Err(BadInput("a car object is missing"));
        }
    }
    Ok(cars)
}

/// What the functions that copy the names write in the car objects (and the names, which
/// a self-test changes).
pub fn regions(cars: &[u32; PLAYERS]) -> Vec<Region> {
    let mut regions = vec![(game_addr(NAMES_RVA) as u32, PLAYERS * NAME_LEN)];
    for &car in cars {
        regions.push((car + CAR_NAME_POINTER as u32, 4));
        regions.push((car + CAR_NAME as u32, NAME_LEN));
    }
    regions
}

/// For the self-tests: names that differ, the last without a NUL in its 0x20 bytes.
pub unsafe fn write_test_names() {
    unsafe {
        for player in 0..PLAYERS {
            let text = format!("player {player} of the test's{}", "!".repeat(player * 3));
            name(player).write_bytes(0, NAME_LEN);
            std::ptr::copy(text.as_ptr(), name(player), text.len().min(NAME_LEN));
        }
    }
}
