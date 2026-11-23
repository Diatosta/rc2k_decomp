//! Unless `rear_wheels_off` is set: `downforce` = (car speed² `downforce_factor` +
//! `downforce_spec`) `gravity` (gravity, by `physics::apply_patches`),
//! `PhysicsTables.forces[4].force[1]` is its negative, `PhysicsTables.forces[4].point[2]` = -1. The
//! car's downward force: its weight plus downforce growing with the speed's square.

use crate::math::x87::square_times_plus_times;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::figures::DOWNFORCE_FACTOR;
use crate::physics::tables_store::PhysicsTables;

pub(super) fn set_downforce(car: &Car, tables: &mut PhysicsTables) {
    if car.rear_wheels_off != 0 {
        return;
    }
    let (load, negative) = square_times_plus_times(
        car.local_velocity.bits_at(2),
        DOWNFORCE_FACTOR,
        tables.downforce_spec,
        tables.gravity,
    );
    tables.downforce = load;
    tables.forces[4].force[1] = negative;
    tables.forces[4].point[2] = -1.0f32;
}
