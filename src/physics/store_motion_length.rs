//! `physics::store_motion_length` (0x43BD2D): `dword_71A19C` = the length of the three
//! floats at `dword_71A170` (`physics::vector_length`, then `fstp`: rounded to a float).
//! Both are the work copy's (+0x198, +0x1C4); that the vector is the car's motion is a
//! reading (it sits next to `g_carSpeed`), not seen.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_43B635` (0x43B644, 0x43B94D), the car's physics step under
//! `sub_4454D2`.

use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::keep_motion_vectors`): the length of the car's motion.
pub fn store_motion_length() {
    crate::race::owner::race()
        .shared
        .car
        .speed
        .set_bits(rounded(vector_length(
            &crate::race::owner::race().shared.car.velocity,
        )));
}
