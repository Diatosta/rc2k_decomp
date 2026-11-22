//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! `checkpoint` against the lead car's (car block 0's), the difference plus one kept when below the
//! smallest so far (unsigned).

/// The gap = (behind: lead - car, else car - lead) + 1; into `best` when below it.
/// Returns the gap, which `drive::start_cars_behind` reads.
pub(super) fn note_gap_at(car_checkpoint: u32, lead: u32, best: &mut u32, behind: bool) -> u32 {
    let mut now = *best;
    let gap = rc2k_core::drive::places::note_gap(car_checkpoint, lead, behind, &mut now);
    *best = now;
    gap
}
