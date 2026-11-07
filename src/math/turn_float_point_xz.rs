//! `math::turn_float_point_xz` (0x41FE74): a point of three floats in eax, ebx, ecx (as
//! dwords) turned in x and z: stored in the scratch point (`g_scratchX`, `Y`, `Z`),
//! `math::turn_scratch_xz` by edx, and loaded back. A sibling of `math::turn_point_xz`
//! without the whole numbers.
//!
//! Takes: x, y, z (floats), the angle
//! Out: eax x', ebx y, ecx z' (floats), edx as `turn_scratch_xz` leaves it; the rest kept
//!      (esi, ebp pushed around); no flags read
//!
//! Callers: `sub_43F136`, `sub_45DA14`, `sub_45DC3E`, `sub_4B7058`.

use crate::math::turn_scratch_xz::{Scratch, turn_scratch_xz, turn_scratch_xz_plain};

/// The Rust version: x', y, z' and the entry left in edx.
pub fn turn_float_point_xz(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    turn_float_with(x, y, z, angle, turn_scratch_xz)
}

/// The converted version (`turn_scratch_xz_plain`).
pub fn turn_float_point_xz_plain(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    turn_float_with(x, y, z, angle, turn_scratch_xz_plain)
}

fn turn_float_with(
    x: u32,
    y: u32,
    z: u32,
    angle: u32,
    turn: fn(&mut Scratch, u32) -> u32,
) -> [u32; 4] {
    let mut point = Scratch { x, y, z };
    let entry = { turn(&mut point, angle) };
    [point.x, point.y, point.z, entry]
}
