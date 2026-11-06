//! `physics::take_asked_gear` (0x43C1E3): `g_car` +0x238 (`unk_71A210`) 0; the gear
//! asked for, bits 8 .. 11 of +0x468 (`unk_71A440`) as a signed nibble, through
//! `physics::allowed_gear` into `dword_614C40`; then `physics::change_gear`.
//!
//! In:  nothing
//! Out: as `change_gear` leaves them; no flags read (the caller returns)
//!
//! Caller: `sub_43C15C`. Refuses what the two calls refuse; what was written by then is put
//! back.

use crate::errors::Refused;
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::car_store::CAR;
use crate::physics::change_gear::change_gear;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;

/// The Rust version.
// Converted: `rc2k_core::physics::gearbox::asked_gear`.
pub fn take_asked_gear(course: &Course) -> Result<(), Refused> {
    let run = || -> Result<(), Refused> {
        CAR.gear_changed.set(0u32);
        let word: u32 = CAR.input_keys.get();
        let asked = rc2k_core::physics::gearbox::asked_gear(word);
        let gear = allowed_gear(course, asked)?;
        PHYSICS.gear_asked.set(gear);
        change_gear(course)
    };

    run()
}
