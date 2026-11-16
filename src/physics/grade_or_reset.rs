//! With `driver_kind` 2, `reset_grades_at` (the work copy's grades reset); otherwise
//! `rate_damage` (its damage made into grades and failures).

use crate::physics::rate_damage::rate_damage;
use crate::physics::reset_car_grades::reset_grades_at;
use crate::physics::store::DamageModel;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

/// `CAR.driver_kind` 2 resets the grades.
pub fn grade_or_reset(damage: &DamageModel, numbers: &Numbers, shared: &RaceShared) {
    if shared.car().driver_kind.get() == 2 {
        reset_grades_at(
            numbers,
            shared.car(),
            &shared.car().suspension_stiffness,
            &shared.car().ride_height,
        );
    } else {
        rate_damage(shared.car(), damage);
    }
}
