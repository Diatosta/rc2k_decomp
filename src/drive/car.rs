//! The current car's work copy: the physics and the drawing work on `g_car` (0x719FD8,
//! 0xFE0 bytes), copied in from the car's block (`g_carBlocks[g_currentCar & 7]`) and back
//! out to it (`drive::load_car`, `drive::store_car`). Fields of the copy met so far are
//! named in IDA by their addresses (`g_carGear` 0x71A1AC, ...; `physics/gearing.rs`).

use crate::errors::Refused;
use crate::physics::car_store::Car;

pub const CAR_LEN: usize = 0xFE0;
const _: () = assert!(core::mem::size_of::<Car>() == CAR_LEN);

/// The current car's block (`g_carBlocks[g_currentCar & 7]`, by its handle), refused when
/// null (the original's `rep movsd` would fault there).
pub fn current_block() -> Result<&'static Car, Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(crate::race::work_store::RACE_WORK.current_car.get());
    let block = car_block(crate::race::work_store::RACE_WORK.car_block(slot as usize))?;
    Ok(Car::at(block))
}

/// Copies `CAR_LEN` bytes, dword by dword upwards as `rep movsd` does (the two never
/// overlap in the game; the order is kept anyway).
pub fn copy_car(from: &Car, to: &Car) {
    crate::pointers::copy_cells((from, 0), (to, 0), CAR_LEN);
}
