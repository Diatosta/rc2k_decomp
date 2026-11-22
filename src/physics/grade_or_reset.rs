//! With `driver_kind` 2, `reset_grades_at` (the work copy's grades reset); otherwise
//! `rate_damage` (its damage made into grades and failures).

use crate::physics::rate_damage::rate_damage;
use crate::physics::reset_car_grades::reset_grades_at;
use crate::physics::store::DamageModel;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

/// `CAR.driver_kind` 2 resets the grades.
pub fn grade_or_reset(damage: &mut DamageModel, numbers: &Numbers, shared: &RaceShared) {
    let mut car = shared.car_mut();
    if car.driver_kind == 2 {
        let (stiffness, ride_height) = (car.suspension_stiffness, car.ride_height);
        reset_grades_at(numbers, &mut car, stiffness, ride_height);
    } else {
        rate_damage(&mut car, damage);
    }
}
