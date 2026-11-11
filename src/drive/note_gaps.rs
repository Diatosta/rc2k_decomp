//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! (esi) checkpoint (+0x98) against the lead car's (car block 0's +0x98, `dword_70F484`),
//! the difference plus one kept when below the smallest so far (unsigned).

use crate::physics::car_store::Car;
use crate::session::Dword;

/// edx = (behind: lead - car, else car - lead) + 1; into `best` when below it.
// Converted: `rc2k_core::drive::places::note_gap`.
/// The gap (edx), which `drive::start_cars_behind` reads.
pub fn note_gap_at(car: &Car, lead: &Dword, best: &Dword, behind: bool) -> u32 {
    let mine: u32 = car.checkpoint.get();
    let theirs: u32 = lead.get();
    let was: u32 = best.get();
    let mut now = was;
    let edx = rc2k_core::drive::places::note_gap(mine, theirs, behind, &mut now);
    if now != was {
        best.set(now);
    }
    edx
}
