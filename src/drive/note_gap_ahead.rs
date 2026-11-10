//! `drive::note_gap_ahead` (0x4373A4): this car's checkpoint less the lead car's, plus one, into
//! `dword_70FA14` (`pace_gaps[0]`) when below it (unsigned): the arcade race's gap in checkpoints to
//! the nearest kept opponent ahead of the lead car (the least over the two kept cars, which
//! `start_cars_behind` starts at -1 = none).
//!
//! Takes: a car block
//! Returns: the difference plus one
//!
//! See `drive/note_gaps.rs`. Caller: `drive::start_cars_behind`.

use crate::drive::note_gaps::note_gap_at;
use crate::physics::car_store::Car;
use crate::race::blocks_store::CAR_BLOCKS;

pub fn note_gap_ahead(car: &Car) -> u32 {
    note_gap_at(
        car,
        &CAR_BLOCKS.cars[0].checkpoint,
        &CAR_BLOCKS.cars[0].pace_gaps[0],
        false,
    )
}
