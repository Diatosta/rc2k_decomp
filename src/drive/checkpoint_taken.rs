//! `drive::checkpoint_taken` (0x4375A5): whether checkpoint eax is no place to start a
//! car: CF when it is within 8 of the stage's start or end (`g_checkpointCount` - 8; the
//! sign of the differences) or `drive::car_near_checkpoint` finds one of the four cars
//! near it (blocks 0x70F3EC, 0x7103CC, 0x7113AC, 0x71238C). Inside `pusha` / `popa`.
//!
//! In:  eax the checkpoint
//! Out: CF; every register kept (ZF not read: `jb`)
//!
//! A `js` is in its first five bytes: `.private_copy` (its four calls relocated) with
//! `.manual_patch`.
//! Caller: `sub_43703E` (0x4371A1, 0x437284).

use crate::drive::car_near_checkpoint::near_at;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::physics::line_places;

/// The Rust core: the four cars at `cars`, the count `count`, the checkpoint
/// `checkpoint` (eax): whether it is taken (the CF).
// Converted: `rc2k_core::physics::line_places::checkpoint_taken`.
pub unsafe fn taken_at(cars: [u32; 4], count: u32, checkpoint: u32) -> bool {
    line_places::checkpoint_taken(checkpoint, count, || {
        cars.iter().any(|&car| unsafe { near_at(car, checkpoint) })
    })
}

/// The Rust version: whether `checkpoint` is taken (the CF; every register kept).
pub unsafe fn checkpoint_taken(checkpoint: u32) -> bool {
    let cars = [0, 1, 2, 3].map(|n| crate::race::blocks_store::block_addr(n) as u32);
    unsafe { taken_at(cars, RACE_VARS.checkpoint_count.bits(), checkpoint) }
}
