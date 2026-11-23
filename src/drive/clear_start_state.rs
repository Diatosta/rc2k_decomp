//! Six of the work copy's start values 0: `driver_kind`
//! (the grades' mode, `physics::grade_or_reset`), `checkpoint`, `engine_sound_speed_int`, `start_share`,
//! `start_blend`, `Car.reserved_44`.

use crate::race::owner_shared::RaceShared;

pub fn clear_start_state(shared: &RaceShared) {
    let mut car = shared.car_mut();
    car.driver_kind = 0;
    car.checkpoint = 0;
    car.engine_sound_speed_int = 0;
    car.start_share = 0.0;
    car.start_blend = 0.0;
    car.reserved_44 = 0;
}
