//! `marks::blend_corner` (0x4D0B4E): one point blended: its factor the record's float at
//! +0x90 + 4 x (the source point's three sign bits, x the highest; kept in
//! `flt_93C6C8`), each of x, y, z base + (source - base) by it, rounded (`fistp`).
//!
//! In:  the record (a car's +0x29C), where the result goes, the source point, the base
//!      point (the original's ebp, esi, edi, edx)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `marks::morph_object_points`.

use crate::marks::store::{Slot, game_write};
use crate::math::x87;

pub const FACTOR_RVA: Slot = Slot::at(0x93C6C8); // flt_93C6C8

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header): `block` the block (the original's ebp), `to` where
/// the point goes (esi), `from` the point blended (edi), `base` the base point (edx).
pub unsafe fn blend_corner(block: u32, to: u32, from: u32, base: u32) {
    unsafe {
        let src = [get(from), get(from + 4), get(from + 8)];
        let signs = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        let f = x87::loaded(get(block + 0x90 + 4 * signs));
        game_write(FACTOR_RVA, f);
        for (c, &s) in src.iter().enumerate() {
            let c = 4 * c as u32;
            let v = x87::int_lerp_to_int(s as i32, get(base + c) as i32, f);
            ((to + c) as *mut i32).write_unaligned(v);
        }
    }
}
