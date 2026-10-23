//! `drive::note_gap_behind` (0x4373C0): the lead car's checkpoint less this car's (esi +0x98), plus one, into `dword_70FA18` when below it (unsigned): by the look of it the nearest car behind the lead (a reading).
//!
//! In:  esi a car block
//! Out: edx the difference plus one; the rest kept; no flags read
//!
//! See `drive/note_gaps.rs`. Caller: `sub_43703E` (0x437364, 0x437390).
//! Self-test: a copy of the original (the lead's checkpoint and the best the test's) on
//! a test car, 7 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::note_gaps::note_gap_at;
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const BEST_RVA: usize = rva(0x70FA18);
const LEAD_RVA: usize = rva(0x70F484); // dword_70F484: car block 0's +0x98

/// The Rust version.
pub unsafe fn note_gap_behind(car: u32) -> Left {
    unsafe {
        note_gap_at(
            car,
            game_addr(LEAD_RVA) as u32,
            game_addr(BEST_RVA) as u32,
            true,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { note_gap_behind(r.esi) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
