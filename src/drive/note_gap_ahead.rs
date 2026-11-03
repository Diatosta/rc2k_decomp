//! `drive::note_gap_ahead` (0x4373A4): this car's checkpoint (esi +0x98) less the lead car's, plus one, into `dword_70FA14` when below it (unsigned): by the look of it the nearest car ahead (a reading).
//!
//! In:  esi a car block
//! Out: edx the difference plus one; the rest kept; no flags read
//!
//! See `drive/note_gaps.rs`. Caller: `sub_43703E` (0x43736B, 0x437397).

use crate::drive::note_gaps::note_gap_at;
use crate::physics::car_store::Car;
use crate::race::blocks_store::CAR_BLOCKS;

/// The Rust version.
pub fn note_gap_ahead(car: &Car) -> u32 {
    note_gap_at(
        car,
        &CAR_BLOCKS.cars[0].checkpoint,
        &CAR_BLOCKS.cars[0].pace_gaps[0],
        false,
    )
}
