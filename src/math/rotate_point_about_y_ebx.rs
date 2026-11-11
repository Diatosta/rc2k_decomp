//! `math::rotate_point_about_y_ebx` (0x41FDFE): `math::rotate_point_about_y` with other registers: x in ebx, y in
//! ecx, z in eax (whole numbers), through the scratch point (`fild`/`fstp` in,
//! `math::rotate_scratch_about_y` by edx, `fld`/`fistp` out); ecx is pushed around, so it comes
//! back as it was.
//!
//! Takes: x, y, z, the angle
//!
//! Callers: `physics::place_camera` (twice).

use crate::math::rotate_point_about_y::rotate_point_about_y;

/// The Rust version: x', z' and the entry left in edx.
pub fn rotate_point_about_y_ebx(x: i32, y: i32, z: i32, angle: u32) -> [u32; 3] {
    let [x, _, z, entry] = rotate_point_about_y(x, y, z, angle);
    [x, z, entry]
}
