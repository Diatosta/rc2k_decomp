//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! `checkpoint` against the lead car's (car block 0's), the difference plus one kept when below the
//! smallest so far (unsigned).

use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;

/// The gap = (behind: lead - car, else car - lead) + 1; into `best` when below it.
/// Returns the gap, which `drive::start_cars_behind` reads.
pub(super) fn note_gap_at(car: &Car, lead: &PlainDword, best: &PlainDword, behind: bool) -> u32 {
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
