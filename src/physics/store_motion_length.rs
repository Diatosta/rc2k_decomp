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

use crate::physics::car_store::CAR;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;

/// The Rust core on the vector at `v` and the length at `out`.
pub fn store_length(v: u32, out: u32) {
    crate::pointers::write_or_end::<u32>(
        out,
        rounded(vector_length(v)),
        "physics::store_motion_length's operand",
    )
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::keep_motion_vectors`).
pub fn store_motion_length() {
    store_length(CAR.velocity[0].addr(), CAR.speed.addr())
}
