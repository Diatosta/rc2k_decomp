//! When the given bit of the car's `parts_present` is set, it is cleared and part 0x50 + 0x14 x
//! `current_car` + bit goes into a race slot (`physics::fill_race_slot`).
//!
//! Callers: `physics::drop_broken_parts`, `physics::drop_hit_parts`. Refuses what
//! `fill_race_slot` refuses (before anything: the bit is cleared after).

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::physics::fill_race_slot::fill_race_slot;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::parts::{part_number, take_bit};

/// Bit `bit` (cl) of the car's parts taken and its part put in a race slot.
// Core: `rc2k_core::physics::parts::{take_bit, part_number}`.
pub fn fill_slot_on_bit(
    numbers: &Numbers,
    work: &RaceWork,
    car: &Car,
    bit: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (_, cleared) = take_bit(car.parts_present.get(), bit);
    let Some(cleared) = cleared else {
        return Ok(());
    };
    let Some(part) = part_number(work.current_car.get(), bit) else {
        return Err(BadInput("a part past 0xFF").into());
    };
    car.parts_present.set(cleared);
    fill_race_slot(numbers, work, car, part, scene)
}
