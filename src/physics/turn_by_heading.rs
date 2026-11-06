//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! In:  esi the point, ebx passed on
//! Out: eax x', ecx z', edx as `turn_scratch_xz` leaves it; the rest kept; no flags read
//!
//! Caller: `sub_45E800` (0x45E8A9).

use crate::math::turn_float_point_xz::turn_float_point_xz;

/// The Rust version: `point` is the three floats (x, y, z); x and z are turned.
pub fn turn_by_heading(point: &[crate::session::Float; 3], ebx: u32) {
    let angle = crate::race::owner::race()
        .shared
        .car
        .car_heading
        .get()
        .wrapping_neg();
    let [x, _, z, _] = turn_float_point_xz(point[0].bits(), ebx, point[2].bits(), angle);
    point[0].set_bits(x);
    point[2].set_bits(z);
}
