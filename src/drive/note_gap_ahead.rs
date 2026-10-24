//! `drive::note_gap_ahead` (0x4373A4): this car's checkpoint (esi +0x98) less the lead car's, plus one, into `dword_70FA14` when below it (unsigned): by the look of it the nearest car ahead (a reading).
//!
//! In:  esi a car block
//! Out: edx the difference plus one; the rest kept; no flags read
//!
//! See `drive/note_gaps.rs`. Caller: `sub_43703E` (0x43736B, 0x437397).

use crate::drive::note_gaps::note_gap_at;
use crate::framework::{game_addr, rva};

const BEST_RVA: usize = rva(0x70FA14);
const LEAD_RVA: usize = rva(0x70F484); // dword_70F484: car block 0's +0x98

/// The Rust version.
pub unsafe fn note_gap_ahead(car: u32) -> u32 {
    unsafe {
        note_gap_at(
            car,
            game_addr(LEAD_RVA) as u32,
            game_addr(BEST_RVA) as u32,
            false,
        )
    }
}
