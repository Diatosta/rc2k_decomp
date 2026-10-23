//! `drive::store_car` (0x460C5D): the work copy back into the current car's block:
//! `g_car` -> `g_carBlocks[g_currentCar & 7]` (0xFE0 bytes, `rep movsd`).
//!
//! In:  nothing (`g_currentCar` 0x71B3F8)
//! Out: eax = `g_currentCar & 7`, ecx = 0, esi = 0x719FD8 + 0xFE0, edi = the block + 0xFE0
//!      (flags: the `and`'s; no caller reads them)
//!
//! See `drive/car.rs`. Callers (17): the race's frame, both halves (`sub_43703E`,
//! `sub_460B2F` under `sub_4454D2`; `sub_454253`, `sub_454265`, `sub_454289`,
//! `sub_446463`, `sub_4608BB` under `sub_4541BF`).
//! Refuses a null block (the original faults).
//! Self-test: a copy of the original whose three variables are the test's, against the
//! Rust core on the same, every slot number's low bits and some high ones; the work copy
//! and all eight blocks compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::car::*;
use crate::errors::Refused;
use crate::screens::harness::Left;

/// The Rust core on the variables at `p`: what it leaves in the registers.
// Converted: `rc2k_core::drive::work_car::{car_slot, car_block}`.
pub unsafe fn store_car_at(p: CarPlaces) -> Result<Left, Refused> {
    unsafe {
        let (slot, block) = p.block()?;
        copy_dwords(p.work, block);
        Ok(left(slot, p.work, block))
    }
}

/// The Rust version, on the game's variables.
pub unsafe fn store_car() -> Result<Left, Refused> {
    unsafe { store_car_at(CarPlaces::game()) }
}
