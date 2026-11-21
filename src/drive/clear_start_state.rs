//! Six of the work copy's start values 0: `driver_kind`
//! (the grades' mode, `physics::grade_or_reset`), `checkpoint`, `engine_sound_speed_int`, `start_share`,
//! `start_blend`, `Car.reserved_44`.

use crate::race::owner_shared::RaceShared;

pub fn clear_start_state(shared: &RaceShared) {
    let car = shared.car();
    car.driver_kind.set(0);
    car.checkpoint.set(0);
    car.engine_sound_speed_int.set(0);
    car.start_share.set(0.0);
    car.start_blend.set(0.0);
    car.reserved_44.set(0);
}
