//! `physics::grade_or_reset` (0x43BA8E): with `driver_kind` 2, `physics::reset_current_car`
//! (the work copy's grades reset); else `physics::rate_current_car` (its damage made into
//! grades and failures).
//!
//! Caller: `physics::set_handling` (0x43EB5D), first.

use crate::physics::rate_damage::rate_damage;
use crate::physics::reset_car_grades::reset_grades_at;
use crate::race::owner::Race;

/// `CAR.driver_kind` 2 resets the grades.
pub fn grade_or_reset(race: &Race) {
    let shared = &race.shared;
    if shared.car.driver_kind.get() == 2 {
        reset_grades_at(
            &race.course.numbers,
            &shared.car,
            &shared.car.suspension_stiffness,
            &shared.car.ride_height,
        );
    } else {
        rate_damage(&race.shared.car, &race.physics.damage);
    }
}
