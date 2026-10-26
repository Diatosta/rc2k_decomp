//! The current car's work copy: the physics and the drawing work on `g_car` (0x719FD8,
//! 0xFE0 bytes), copied in from the car's block (`g_carBlocks[g_currentCar & 7]`) and back
//! out to it (`drive::load_car`, `drive::store_car`). Fields of the copy met so far are
//! named in IDA by their addresses (`g_carGear` 0x71A1AC, ...; `physics/gearing.rs`).

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;

pub const CAR_LEN: usize = 0xFE0;

/// Where a copy of the car functions finds its variables: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct CarPlaces {
    pub current: u32,
    pub blocks: u32,
    pub work: u32,
}

impl CarPlaces {
    pub fn game() -> Self {
        Self {
            current: crate::race::work_store::RACE_WORK.current_car.addr() as u32,
            blocks: crate::pointers::game_address(
                crate::race::work_store::RACE_WORK.car_blocks.as_ptr(),
            ),
            work: crate::physics::car_store::car_addr() as u32,
        }
    }

    /// The current car's slot (0 .. 7) and its block, refused when the block is null
    /// (the original's `rep movsd` would fault there).
    pub fn block(&self) -> Result<(u32, u32), Refused> {
        use rc2k_core::drive::work_car::{car_block, car_slot};
        let slot = car_slot(GamePtr::new(self.current).read("the current car")?);
        let block =
            car_block(GamePtr::new(self.blocks.wrapping_add(4 * slot)).read("a car's block")?)?;
        Ok((slot, block))
    }
}

/// Copies `CAR_LEN` bytes, dword by dword upwards as `rep movsd` does (the two never
/// overlap in the game; the order is kept anyway).
pub fn copy_dwords(from: u32, to: u32) -> Result<(), BadInput> {
    GamePtr::new(from).copy_to(GamePtr::new(to), CAR_LEN, "a car's block")
}
