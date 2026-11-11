//! `physics::put_back_motion_vectors` (0x43BD8B): the three dwords at `dword_614BCC` ..
//! back into the work car's motion (`dword_71A170` .., `g_car` +0x198) and those at
//! `dword_614BDC` .. into its spin (`dword_71A15C` .., +0x184), eax pushed around; then
//! `physics::car_frame_motion`. The undoing of `physics::keep_motion_vectors`.
//!
//! Caller: `physics::car_physics_step` (0x43B8F1, before `physics::limit_motion`).

use crate::physics::car_frame_motion::car_frame_motion;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;

/// The kept motion (`kept_vectors` 0 .. 2) and spin (4 .. 6) back, the motion turned into the car's
/// frame.
pub fn put_back_motion_vectors(car: &Car, tables: &PhysicsTables) {
    for k in 0..3 {
        car.velocity[k].set_bits(tables.kept_vectors[k].get());
        car.spin[k].set_bits(tables.kept_vectors[4 + k].get());
    }
    car_frame_motion(&car.velocity, &car.car_inverse_matrix, &car.local_velocity);
}
