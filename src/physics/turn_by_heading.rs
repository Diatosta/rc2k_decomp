//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! Takes: the point, passed on
//!
//! Caller: `physics::point_hits_car` (0x45E8A9).

use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::race::owner_shared::Shared;

/// `point` is the three floats (x, y, z); x and z are turned.
pub fn turn_by_heading(shared: &Shared, point: &[crate::session::Float; 3], ebx: u32) {
    let angle = shared.car.car_heading.get().wrapping_neg();
    let [x, _, z, _] = turn_float_point_xz(point[0].bits(), ebx, point[2].bits(), angle);
    point[0].set_bits(x);
    point[2].set_bits(z);
}
