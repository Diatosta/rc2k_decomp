//! `math::turn_point_xz_ebx` (0x41FDFE): `math::turn_point_xz` with other registers: x in ebx, y in
//! ecx, z in eax (whole numbers), through the scratch point (`fild`/`fstp` in,
//! `math::turn_scratch_xz` by edx, `fld`/`fistp` out); ecx is pushed around, so it comes
//! back as it was.
//!
//! In:  ebx x, ecx y, eax z, edx the angle
//! Out: ebx x', eax z', edx as `turn_scratch_xz` leaves it; the rest kept (esi, ebp
//!      pushed too); no flags read
//!
//! Callers: `sub_438E5A` (twice).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::math::turn_point_xz::turn_point_xz;

/// The Rust version: x', z' and the entry left in edx.
pub unsafe fn turn_point_xz_ebx(x: i32, y: i32, z: i32, angle: u32) -> [u32; 3] {
    let [x, _, z, entry] = unsafe { turn_point_xz(x, y, z, angle) };
    [x, z, entry]
}
