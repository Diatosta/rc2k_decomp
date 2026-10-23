//! `math::turn_scratch_xz` (0x41FEF8): the scratch point's x and z (floats at
//! `dword_5C5F50` and `dword_5C5F4C`) turned by an angle.
//!
//! In:  edx = the angle (a dword's turn is 2^32: `g_sines`/`g_cosines` entry (edx +
//!      0x80000) >> 20)
//! Out: edx = the entry; the rest preserved
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 as the original (`math::x87::
//! turn_in_place`). Six callers, `math::turn_point_xz` and its three siblings.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::{game_addr, game_read, game_write, rva};

pub const Z_RVA: usize = rva(0x5C5F4C); // dword_5C5F4C
pub const X_RVA: usize = rva(0x5C5F50); // dword_5C5F50
pub const Y_RVA: usize = rva(0x5C5F54); // dword_5C5F54
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines

/// `math::turn_scratch_xz` exactly as the original (the x87, at whatever precision it is
/// set to): what the Rust callers in modules not yet converted use, so that their own
/// comparisons stay exact. The entry left in edx.
pub unsafe fn turn_scratch_xz(angle: u32) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    unsafe {
        x87_turn(entry);
    }
    entry
}

unsafe fn x87_turn(entry: u32) {
    let a = |r: usize| game_addr(r + 4 * entry as usize);
    unsafe {
        crate::math::x87::turn_in_place(
            a(SINES_RVA) as *const u32,
            a(COSINES_RVA) as *const u32,
            game_addr(X_RVA) as *mut u32,
            game_addr(Z_RVA) as *mut u32,
        )
    };
}

/// The converted version (`rc2k_core::math::geometry`, plain floats): the hook's, and what
/// converted callers use. The entry left in edx.
pub unsafe fn turn_scratch_xz_plain(angle: u32) -> u32 {
    use rc2k_core::math::geometry;
    let entry = geometry::turn_entry(angle);
    unsafe {
        let float = |rva: usize| f32::from_bits(game_read(rva));
        let (x, z) = geometry::turn_xz(
            float(X_RVA),
            float(Z_RVA),
            float(SINES_RVA + 4 * entry as usize),
            float(COSINES_RVA + 4 * entry as usize),
        );
        game_write(X_RVA, x.to_bits());
        game_write(Z_RVA, z.to_bits());
    }
    entry
}
