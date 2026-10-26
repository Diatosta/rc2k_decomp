//! `math::turn_point_xz` (0x41FD12): a point of whole numbers turned in x and z.
//!
//! In:  eax = x, ebx = y, ecx = z, edx = the angle
//! Out: eax = x', ecx = z' (as `math::turn_scratch_xz` makes them, rounded as the x87
//!      rounds), edx as `turn_scratch_xz` leaves it; the rest preserved (ebx too: the
//!      original loads y into it, then its `pop ebx` puts the caller's back)
//!
//! The three go through the scratch point (`dword_5C5F50` x, `dword_5C5F54` y,
//! `dword_5C5F4C` z; a local since 2026-10-24: `turn_scratch_xz::Scratch`), left there as
//! whole numbers. 29 callers (the map, the cameras, ..).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::math::turn_scratch_xz::{Scratch, turn_scratch_xz};

/// `math::turn_point_xz`: x', y (as it comes back through a float, left only in the
/// scratch point), z' and the entry left in edx.
pub fn turn_point_xz(x: i32, y: i32, z: i32, angle: u32) -> [u32; 4] {
    turn_point_with(x, y, z, angle, turn_scratch_xz)
}

fn turn_point_with(
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
    turn: fn(&mut Scratch, u32) -> u32,
) -> [u32; 4] {
    // Converted: `rc2k_core::math::geometry` (`fild` / `fstp` is `as f32`, `fistp` below).
    let mut point = Scratch {
        x: (x as f32).to_bits(),
        y: (y as f32).to_bits(),
        z: (z as f32).to_bits(),
    };
    let entry = turn(&mut point, angle);
    let back = |v: u32| rc2k_core::math::geometry::fistp(f32::from_bits(v)) as u32;
    [back(point.x), back(point.y), back(point.z), entry]
}
