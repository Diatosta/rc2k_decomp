//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! In:  esi the point, ebx passed on
//! Out: eax x', ecx z', edx as `turn_scratch_xz` leaves it; the rest kept; no flags read
//!
//! Caller: `sub_45E800` (0x45E8A9).

use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::physics::car_store::CAR;

/// The Rust version.
pub fn turn_by_heading(point: u32, ebx: u32) {
    use crate::pointers::{read_or_end, write_or_end};
    let what = "a point to turn";
    let angle = CAR.car_heading.get().wrapping_neg();
    let [x, _, z, _] = turn_float_point_xz(
        read_or_end::<u32>(point, what),
        ebx,
        read_or_end::<u32>(point + 8, what),
        angle,
    );
    write_or_end(point, x, what);
    write_or_end(point + 8, z, what);
}
