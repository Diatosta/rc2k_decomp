//! For cars of kind (`car_kind`) below 0x17, four parts in turn (`physics::car_word_bits` bits
//! 5, 9, 0xE, 0x19 of the car flags; their timers from `wheel_hit_damage`): the first whose bit
//! is set and timer not below 100 loses its part when its bit of `parts_present` (5, 6, 7, 8)
//! is still set (`physics::fill_slot_on_bit`, the timer 256) and it ends there; a bit of
//! `parts_present` already clear ends it too. A reading: hit parts coming off.
//!
//! Refuses what `fill_slot_on_bit` refuses; what was written is put back.

use crate::errors::Refused;
use crate::math::x87;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::parts::{self, DROP_TESTS, Hit, HitTest};

/// The test that acts (`rc2k_core::physics::parts::hit_tests`): the first whose bit of the car's
/// flags is set and whose timer is not below 100 (and whose part is still on): its number in
/// `tests`.
pub fn hit_pick(
    noise_word: u32,
    parts_present: u32,
    numbers: &Numbers,
    tests: &[HitTest; 4],
    timers: &[f32; 4],
) -> Option<usize> {
    let (_, hit) = parts::hit_tests(
        tests,
        noise_word,
        |k| timers.at(k),
        || numbers.hundred.get(),
        || parts_present,
    );
    match hit {
        Hit::Act(k) => Some(k),
        _ => None,
    }
}

// Core: `rc2k_core::physics::parts::{hit_tests, DROP_TESTS}`.
pub(super) fn drop_hit_parts(
    numbers: &Numbers,
    work: &mut RaceWork,
    car: &mut Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if car.car_kind >= 0x17 {
        return Ok(());
    }
    if let Some(k) = hit_pick(
        car.noise_word,
        car.parts_present,
        numbers,
        &DROP_TESTS,
        &car.wheel_hit_damage,
    ) {
        fill_slot_on_bit(numbers, work, car, DROP_TESTS[k].1, scene)?;
        car.wheel_hit_damage[k] = x87::loaded(numbers.k256);
    }
    Ok(())
}
