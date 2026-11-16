//! The three dwords of `kept_vectors` go back into the work car's `velocity`, and the following
//! three into its `spin`; then `physics::car_frame_motion` runs. The undoing of
//! `physics::keep_motion_vectors`.

use crate::physics::car_frame_motion::car_frame_motion;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// The kept motion (`kept_vectors` 0 .. 2) and spin (4 .. 6) back, the motion turned into the car's
/// frame.
pub fn put_back_motion_vectors(car: &Car, tables: &PhysicsTables) {
    for k in 0..3 {
        car.velocity[k].set(tables.kept_vectors[k].get());
        car.spin[k].set(tables.kept_vectors[4 + k].get());
    }
    car_frame_motion(&car.velocity, &car.car_inverse_matrix, &car.local_velocity);
}
