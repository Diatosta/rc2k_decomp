//! `drive::load_car` (0x460C40): the current car's block into the work copy:
//! `g_carBlocks[g_currentCar & 7]` -> `g_car` (0xFE0 bytes, `rep movsd`).
//!
//! In:  nothing (`g_currentCar` 0x71B3F8)
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = the block + 0xFE0, edi = 0x719FD8 + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (18): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_45422F`, `sub_454241`, `sub_454277`,
//! `sub_446463`, `sub_45754C`, `sub_4608BB`, `sub_460C7A` under `sub_4541BF`).
//! Refuses a null block (the original faults).
//! Self-test: a copy of the original whose three variables are the test's (`g_car` is used
//! all through a frame), against the Rust core on the same, every slot number's low bits
//! and some high ones.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::screens::harness::Left;

/// The Rust core on the variables at `p`: what it leaves in the registers.
// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub unsafe fn load_car_at(p: CarPlaces) -> Result<Left, Refused> {
    unsafe {
        let (slot, block) = p.block()?;
        copy_dwords(block, p.work);
        Ok(left(slot, block, p.work))
    }
}

/// The Rust version, on the game's variables.
pub unsafe fn load_car() -> Result<Left, Refused> {
    unsafe { load_car_at(CarPlaces::game()) }
}
