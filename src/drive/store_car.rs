//! `drive::store_car` (0x460C5D): the work copy back into the current car's block: `g_car` ->
//! `g_carBlocks[g_currentCar & 7]` (0xFE0 bytes, `rep movsd`).
//!
//! Works on `g_currentCar` 0x71B3F8.
//!
//! See `drive/car.rs`. Callers (17): the race's frame, both halves (`drive::start_cars_behind`,
//! `drive::replay_step` under `drive::race_step`; 0x454253, 0x454265, 0x454289, `race::start_cars`,
//! `drive::race_cars_step` under `drive::race_frame`). Refuses a null block (the original faults).

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub fn store_car(shared: &RaceShared) -> Result<(), Refused> {
    copy_car(shared.car(), current_block(shared)?);

    Ok(())
}
