//! The current car's work copy: the physics and the drawing work on `g_car` (0x719FD8,
//! 0xFE0 bytes), copied in from the car's block (`g_carBlocks[g_currentCar & 7]`) and back
//! out to it (`drive::load_car`, `drive::store_car`). Fields of the copy met so far are
//! named in IDA by their addresses (`g_carGear` 0x71A1AC, ...; `physics/gearing.rs`).

use crate::errors::Refused;
use crate::framework::{game_addr, rva};

pub const CAR_RVA: usize = rva(0x719FD8); // g_car: the current car's work copy
pub const CAR_LEN: usize = 0xFE0;
pub const CURRENT_CAR_RVA: usize = rva(0x71B3F8); // g_currentCar: its number (& 7)
pub use crate::race::objects::CAR_BLOCKS_RVA; // g_carBlocks: eight block pointers

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
            current: game_addr(CURRENT_CAR_RVA) as u32,
            blocks: game_addr(CAR_BLOCKS_RVA) as u32,
            work: game_addr(CAR_RVA) as u32,
        }
    }

    /// The current car's slot (0 .. 7) and its block, refused when the block is null
    /// (the original's `rep movsd` would fault there).
    pub unsafe fn block(&self) -> Result<(u32, u32), Refused> {
        use rc2k_core::drive::work_car::{car_block, car_slot};
        unsafe {
            let slot = car_slot((self.current as *const u32).read_unaligned());
            let block = car_block(((self.blocks + 4 * slot) as *const u32).read_unaligned())?;
            Ok((slot, block))
        }
    }
}

/// Copies `CAR_LEN` bytes, dword by dword upwards as `rep movsd` does (the two never
/// overlap in the game; the order is kept anyway).
pub unsafe fn copy_dwords(from: u32, to: u32) {
    unsafe {
        for k in 0..(CAR_LEN as u32 / 4) {
            let v = ((from + 4 * k) as *const u32).read_unaligned();
            ((to + 4 * k) as *mut u32).write_unaligned(v);
        }
    }
}
