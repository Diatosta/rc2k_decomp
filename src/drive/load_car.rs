//! `drive::load_car` (0x460C40): the current car's block into the work copy:
//! `g_carBlocks[g_currentCar & 7]` -> `g_car` (0xFE0 bytes, `rep movsd`).
//!
//! Works on `g_currentCar` 0x71B3F8.
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = the block + 0xFE0, edi = 0x719FD8 + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (18): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_45422F`, `sub_454241`, `sub_454277`, `sub_446463`,
//! `sub_45754C`, `sub_4608BB`, `sub_460C7A` under `sub_4541BF`). Refuses a null block (the
//! original faults).

use crate::drive::car::*;
use crate::errors::Refused;
use crate::race::owner_shared::Shared;

/// The Rust version.
// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub fn load_car(shared: &Shared) -> Result<(), Refused> {
    copy_car(current_block(shared)?, &shared.car);

    Ok(())
}
