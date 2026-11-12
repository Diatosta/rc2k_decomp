//! `physics::drop_hit_parts` (0x4410C8): for cars of kind (`car_kind`) below 0x17, four parts in
//! turn (`physics::test_car_word` bits 5, 9, 0xE, 0x19 of the car flags; their timers from
//! `wheel_hit_damage`): the first whose bit is set and timer not below 100 loses its part when its
//! bit of `parts_present` (5, 6, 7, 8) is still set (`physics::fill_slot_on_bit`, the timer 256) and
//! it ends there; a bit of `parts_present` already clear ends it too. A reading: hit parts coming off.
//!
//! Caller: `physics::hit_part_checks`. Refuses what `fill_slot_on_bit` refuses; what was
//! written is put back.

use crate::errors::Refused;
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::parts::{self, DROP_TESTS, Hit, HitTest};

/// The four tests' timers.
pub type Timers = [crate::session::PlainFloat; 4];

/// The shared steps (`rc2k_core::physics::parts::hit_tests`): `act` the bit's step (given the bit).
pub fn hit_tests(
    car: &Car,
    numbers: &Numbers,
    tests: &[HitTest; 4],
    timers: &Timers,
    act: &mut dyn FnMut(u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let word = car.noise_word.get();
    let (_, hit) = parts::hit_tests(
        tests,
        word,
        |k| timers[k].get(),
        || numbers.hundred.get(),
        || car.parts_present.get(),
    );
    if let Hit::Act(k) = hit {
        act(tests[k].1)?;
        timers[k].set(x87::loaded(&numbers.k256));
    }
    Ok(())
}

// Converted: `rc2k_core::physics::parts::{hit_tests, DROP_TESTS}`.
pub fn drop_hit_parts(
    numbers: &Numbers,
    work: &RaceWork,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if car.car_kind.get() >= 0x17 {
        return Ok(());
    }
    hit_tests(
        car,
        numbers,
        &DROP_TESTS,
        &car.wheel_hit_damage,
        &mut |bit| fill_slot_on_bit(numbers, work, car, bit, scene),
    )
}
