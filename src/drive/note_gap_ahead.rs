//! This car's checkpoint less the lead car's, plus one, into
//! `pace_gaps[0]` when below it (unsigned): the arcade race's gap in checkpoints to
//! the nearest kept opponent ahead of the lead car (the least over the two kept cars, which
//! `start_cars_behind` starts at -1 = none).
//!
//! See `drive/note_gaps.rs`.

use crate::drive::note_gaps::note_gap_at;
use crate::physics::car_store::Car;
use crate::race::owner::Race;

pub fn note_gap_ahead(race: &Race, car: &Car) -> u32 {
    note_gap_at(
        car,
        &race.car_blocks().cars[0].checkpoint,
        &race.car_blocks().cars[0].pace_gaps[0],
        false,
    )
}
