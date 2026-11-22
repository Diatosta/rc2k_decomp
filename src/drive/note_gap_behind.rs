use crate::drive::note_gaps::note_gap_at;
use crate::race::owner::Race;

/// The gap of the car whose checkpoint is `checkpoint` behind the lead car, kept in the lead's
/// `pace_gaps[1]` when it is below it.
pub fn note_gap_behind(race: &Race, checkpoint: u32) -> u32 {
    let mut blocks = race.car_blocks_mut();
    let lead = &mut blocks.cars[0];
    note_gap_at(checkpoint, lead.checkpoint, &mut lead.pace_gaps[1], true)
}
