//! `physics::drop_hit_parts` (0x4410C8): for cars of kind (+0x24) below 0x17, four parts
//! in turn (`physics::test_car_word` bits 5, 9, 0xE, 0x19 of +0xC; their timers +0x2F8 ..
//! +0x304): the first whose bit is set and timer not below 100 (`flt_5C5D14`) loses its
//! part when its bit of +0x298 (5, 6, 7, 8) is still set (`physics::fill_slot_on_bit`, the
//! timer 256) and it ends there; a bit of +0x298 already clear ends it too. A reading:
//! hit parts coming off.
//!
//! In:  nothing
//! Out: as the calls leave them; no flags read (the caller calls on)
//!
//! Caller: `physics::hit_part_checks`. Refuses what `fill_slot_on_bit` refuses; what was
//! written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use crate::race::owner::Race;
use rc2k_core::physics::parts::{self, DROP_TESTS, Hit, HitTest};

/// The four tests' timers.
pub type Timers = [crate::session::Float; 4];

/// The shared steps (`rc2k_core::physics::parts::hit_tests`): `act` the bit's step (with
/// the bit, the original's ecx). The registers left are read by none of the callers.
pub fn hit_tests(
    race: &Race,
    tests: &[HitTest; 4],
    timers: &Timers,
    act: &mut dyn FnMut(u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let word = race.shared.car.noise_word.get();
    let (_, hit) = parts::hit_tests(
        tests,
        word,
        |k| timers[k].get(),
        || f32::from_bits(race.course.numbers.hundred.bits()),
        || race.shared.car.parts_present.get(),
    );
    if let Hit::Act(k) = hit {
        act(tests[k].1)?;
        timers[k].set_bits(x87::loaded(race.course.numbers.k256.bits()));
    }
    Ok(())
}

/// The Rust version.
// Converted: `rc2k_core::physics::parts::{hit_tests, DROP_TESTS}`.
pub fn drop_hit_parts(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if race.shared.car.car_kind.get() >= 0x17 {
        return Ok(());
    }
    hit_tests(
        race,
        &DROP_TESTS,
        &race.shared.car.wheel_hit_damage,
        &mut |bit| fill_slot_on_bit(race, bit, scene),
    )
}
