//! `drive::load_car` (0x460C40): the current car's block into the work copy:
//! `g_carBlocks[g_currentCar & 7]` -> `g_car` (0xFE0 bytes, `rep movsd`).
//!
//! Works on `g_currentCar` 0x71B3F8.
//!
//! See `drive/car.rs`. Callers (18): the race's frame, both halves (`drive::start_cars_behind`,
//! `drive::replay_step` under `drive::race_step`; 0x45422F, 0x454241, 0x454277, `race::start_cars`,
//! `race::cast_car_shadows`, `drive::race_cars_step`, `race::load_first_car` under
//! `drive::race_frame`). Refuses a null block (the original faults).

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::RaceShared;

// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub fn load_car(shared: &RaceShared) -> Result<(), Refused> {
    copy_car(current_block(shared)?, shared.car());

    Ok(())
}
