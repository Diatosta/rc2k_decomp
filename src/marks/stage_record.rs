//! `marks::stage_record` (0x4D2DE0): `marks::run_frame`'s pick of the record of
//! `dword_64165C` among `off_606320`'s 0x34 (0x5C bytes each; by their count and
//! `sub_4616AA`'s clamp to 0x33, the stages: a reading).
//!
//! In:  nothing
//! Out: eax the number, esi its record
//!
//! One caller, `marks::run_frame`. Refuses a number past the table (unsigned). Self-test:
//! the original and the Rust version on numbers 0, 1, 0x1F and 0x33 and the game's own;
//! put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

const NUMBER_RVA: usize = rva(0x64165C); // dword_64165C
const RECORDS_RVA: usize = rva(0x606320); // off_606320: 0x34 pointers

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::race::stage_record`
pub unsafe fn stage_record(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(NUMBER_RVA);
        let n = rc2k_core::marks::race::stage_record(g.eax)?;
        g.esi = game_read(RECORDS_RVA + 4 * n);
    }
    Ok(g)
}
