//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! (esi) checkpoint (+0x98) against the lead car's (car block 0's +0x98, `dword_70F484`),
//! the difference plus one kept when below the smallest so far (unsigned).

use crate::framework::EDX;
use crate::screens::harness::Left;

/// edx = (behind: lead - car, else car - lead) + 1; into `best` when below it.
// Converted: `rc2k_core::drive::places::note_gap`.
pub unsafe fn note_gap_at(car: u32, lead: u32, best: u32, behind: bool) -> Left {
    unsafe {
        let mine = ((car + 0x98) as *const u32).read_unaligned();
        let theirs = (lead as *const u32).read_unaligned();
        let b = best as *mut u32;
        let was = b.read_unaligned();
        let mut now = was;
        let edx = rc2k_core::drive::places::note_gap(mine, theirs, behind, &mut now);
        if now != was {
            b.write_unaligned(now);
        }
        Left::nothing().with(EDX, edx)
    }
}
