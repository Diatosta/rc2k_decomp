//! `drive::note_gap_behind` (0x4373C0): the lead car's checkpoint less this car's (esi +0x98), plus one, into `dword_70FA18` when below it (unsigned): by the look of it the nearest car behind the lead (a reading).
//!
//! In:  esi a car block
//! Out: edx the difference plus one; the rest kept; no flags read
//!
//! See `drive/note_gaps.rs`. Caller: `sub_43703E` (0x437364, 0x437390).

use crate::drive::note_gaps::note_gap_at;
use crate::race::blocks_store::CAR_BLOCKS;

/// The Rust version.
pub unsafe fn note_gap_behind(car: u32) -> u32 {
    unsafe {
        note_gap_at(
            car,
            CAR_BLOCKS.cars[0].checkpoint.addr() as u32,
            CAR_BLOCKS.cars[0].pace_gaps[1].addr() as u32,
            true,
        )
    }
}
