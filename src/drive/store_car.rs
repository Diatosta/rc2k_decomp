//! `drive::store_car` (0x460C5D): the work copy back into the current car's block:
//! `g_car` -> `g_carBlocks[g_currentCar & 7]` (0xFE0 bytes, `rep movsd`).
//!
//! In:  nothing (`g_currentCar` 0x71B3F8)
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = 0x719FD8 + 0xFE0, edi = the block + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (17): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_454253`, `sub_454265`, `sub_454289`, `sub_446463`,
//! `sub_4608BB` under `sub_4541BF`). Refuses a null block (the original faults).

use crate::drive::car::*;
use crate::errors::Refused;

/// The Rust version.
// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub fn store_car() -> Result<(), Refused> {
    copy_car(&crate::physics::car_store::CAR, current_block()?)?;
    Ok(())
}
