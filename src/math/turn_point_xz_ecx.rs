//! `math::turn_point_xz_ecx` (0x41FD88): `math::turn_point_xz` with other registers: x in ecx, y in
//! eax, z in ebx (whole numbers), through the scratch point (`fild`/`fstp` in,
//! `math::turn_scratch_xz` by edx, `fld`/`fistp` out); eax is pushed around, so it comes
//! back as it was.
//!
//! Takes: x, y, z, the angle
//!
//! Callers: `race::place_view`, `physics::place_camera` (twice).

use crate::math::turn_point_xz::turn_point_xz;

/// The Rust version: x', z' and the entry left in edx.
pub fn turn_point_xz_ecx(x: i32, y: i32, z: i32, angle: u32) -> [u32; 3] {
    let [x, _, z, entry] = turn_point_xz(x, y, z, angle);
    [x, z, entry]
}
