//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! (esi) checkpoint (+0x98) against the lead car's (car block 0's +0x98, `dword_70F484`),
//! the difference plus one kept when below the smallest so far (unsigned).

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::session::Dword;

/// edx = (behind: lead - car, else car - lead) + 1; into `best` when below it.
// Converted: `rc2k_core::drive::places::note_gap`.
/// The gap (edx), which `drive::start_cars_behind` reads.
pub fn note_gap_at(car: u32, lead: &Dword, best: &Dword, behind: bool) -> Result<u32, BadInput> {
    let mine: u32 = GamePtr::new(car.wrapping_add(0x98)).read("a car's block")?;
    let theirs: u32 = lead.bits();
    let was: u32 = best.bits();
    let mut now = was;
    let edx = rc2k_core::drive::places::note_gap(mine, theirs, behind, &mut now);
    if now != was {
        best.set_bits(now);
    }
    Ok(edx)
}
