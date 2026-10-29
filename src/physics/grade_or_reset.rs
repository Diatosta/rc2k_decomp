//! `physics::grade_or_reset` (0x43BA8E): with `dword_71A234` 2, `physics::reset_current_car`
//! (the work copy's grades reset); else `physics::rate_current_car` (its damage made into
//! grades and failures).
//!
//! In:  nothing
//! Out: as the branch taken leaves them; no flags read
//!
//! Caller: `physics::set_handling` (0x43EB5D), first.

use crate::physics::car_store::{CAR, car_addr};
use crate::physics::rate_damage::rate_damage;
use crate::physics::reset_car_grades::reset_grades_at;

/// The Rust version: `CAR.driver_kind` 2 resets the grades.
pub fn grade_or_reset() {
    if CAR.driver_kind.get() == 2 {
        reset_grades_at(car_addr(), &CAR.suspension_stiffness, &CAR.ride_height);
    } else {
        rate_damage();
    }
}
