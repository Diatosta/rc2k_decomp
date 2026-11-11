//! `physics::take_asked_gear` (0x43C1E3): `g_car` +0x238 (`gear_changed`) 0; the gear
//! asked for, bits 8 .. 11 of +0x468 (`input_keys`) as a signed nibble, through
//! `physics::allowed_gear` into `gear_asked`; then `physics::change_gear`.
//!
//! Caller: `physics::gearbox_step`. Refuses what the two calls refuse; what was written by then is
//! put back.

use crate::errors::Refused;
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::change_gear::change_gear;
use crate::race::owner::Race;

// Converted: `rc2k_core::physics::gearbox::asked_gear`.
pub fn take_asked_gear(race: &Race) -> Result<(), Refused> {
    let run = || -> Result<(), Refused> {
        race.shared.car.gear_changed.set(0u32);
        let word: u32 = race.shared.car.input_keys.get();
        let asked = rc2k_core::physics::gearbox::asked_gear(word);
        let gear = allowed_gear(&race.shared.car, &race.course.numbers, asked)?;
        race.shared.physics_tables.gear_asked.set(gear);
        change_gear(race)
    };

    run()
}
