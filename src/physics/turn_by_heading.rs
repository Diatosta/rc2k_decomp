//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! In:  esi the point, ebx passed on
//! Out: eax x', ecx z', edx as `turn_scratch_xz` leaves it; the rest kept; no flags read
//!
//! Caller: `sub_45E800` (0x45E8A9).

use crate::framework::{game_read, rva};
use crate::math::turn_float_point_xz::turn_float_point_xz;

const HEADING_RVA: usize = rva(0x71A998); // g_car + 0x9C0

/// The Rust version.
pub unsafe fn turn_by_heading(point: u32, ebx: u32) {
    unsafe {
        let p = point as *mut u32;
        let angle = game_read::<u32>(HEADING_RVA).wrapping_neg();
        let [x, _, z, _] =
            turn_float_point_xz(p.read_unaligned(), ebx, p.add(2).read_unaligned(), angle);
        p.write_unaligned(x);
        p.add(2).write_unaligned(z);
    }
}
