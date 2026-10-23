//! `math::turn_point_xz` (0x41FD12): a point of whole numbers turned in x and z.
//!
//! In:  eax = x, ebx = y, ecx = z, edx = the angle
//! Out: eax = x', ecx = z' (as `math::turn_scratch_xz` makes them, rounded as the x87
//!      rounds), edx as `turn_scratch_xz` leaves it; the rest preserved (ebx too: the
//!      original loads y into it, then its `pop ebx` puts the caller's back)
//!
//! The three go through the scratch point (`dword_5C5F50` x, `dword_5C5F54` y,
//! `dword_5C5F4C` z), left there as whole numbers. 29 callers (the map, the cameras, ..).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::{game_read, game_write};
use crate::math::turn_scratch_xz::{X_RVA, Y_RVA, Z_RVA, turn_scratch_xz};

/// `math::turn_point_xz`: x', y (as it comes back through a float, left only in the
/// scratch point), z' and the entry left in edx.
pub unsafe fn turn_point_xz(x: i32, y: i32, z: i32, angle: u32) -> [u32; 4] {
    unsafe { turn_point_with(x, y, z, angle, turn_scratch_xz) }
}

unsafe fn turn_point_with(
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
    turn: unsafe fn(u32) -> u32,
) -> [u32; 4] {
    unsafe {
        // Converted: `rc2k_core::math::geometry` (`fild` / `fstp` is `as f32`, `fistp` below).
        game_write(X_RVA, (x as f32).to_bits());
        game_write(Y_RVA, (y as f32).to_bits());
        game_write(Z_RVA, (z as f32).to_bits());
        let entry = turn(angle);
        let back = |r: usize| {
            let v = rc2k_core::math::geometry::fistp(f32::from_bits(game_read(r))) as u32;
            game_write(r, v);
            v
        };
        let (x, y, z) = (back(X_RVA), back(Y_RVA), back(Z_RVA));
        [x, y, z, entry]
    }
}
