//! `physics::put_back_motion_vectors` (0x43BD8B): the three dwords at `dword_614BCC` ..
//! back into the work car's motion (`dword_71A170` .., `g_car` +0x198) and those at
//! `dword_614BDC` .. into its spin (`dword_71A15C` .., +0x184), eax pushed around; then
//! `physics::car_frame_motion`. The undoing of `physics::keep_motion_vectors`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B8F1, before `sub_43F7AC`).

use crate::physics::car_frame_motion::car_frame_motion;

/// The Rust version: the kept motion (`kept_vectors` 0 .. 2) and spin (4 .. 6) back, the
/// motion turned into the car's frame.
pub fn put_back_motion_vectors() {
    for k in 0..3 {
        crate::race::owner::race().shared.car.velocity[k].set_bits(
            crate::race::owner::race()
                .shared
                .physics_tables
                .kept_vectors[k]
                .get(),
        );
        crate::race::owner::race().shared.car.spin[k].set_bits(
            crate::race::owner::race()
                .shared
                .physics_tables
                .kept_vectors[4 + k]
                .get(),
        );
    }
    car_frame_motion(
        &crate::race::owner::race().shared.car.velocity,
        &crate::race::owner::race().shared.car.car_inverse_matrix,
        &crate::race::owner::race().shared.car.local_velocity,
    );
}
