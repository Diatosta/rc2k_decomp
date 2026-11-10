//! `drive::note_gap_behind` (0x4373C0): the lead car's checkpoint less this car's, plus one, into
//! `dword_70FA18` when below it (unsigned): by the look of it the nearest car behind the lead (a
//! reading).
//!
//! Takes: a car block
//! Returns: the difference plus one
//!
//! See `drive/note_gaps.rs`. Caller: `drive::start_cars_behind`.

use crate::drive::note_gaps::note_gap_at;
use crate::physics::car_store::Car;
use crate::race::blocks_store::CAR_BLOCKS;

pub fn note_gap_behind(car: &Car) -> u32 {
    note_gap_at(
        car,
        &CAR_BLOCKS.cars[0].checkpoint,
        &CAR_BLOCKS.cars[0].pace_gaps[1],
        true,
    )
}
