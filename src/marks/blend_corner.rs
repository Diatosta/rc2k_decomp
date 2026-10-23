//! `marks::blend_corner` (0x4D0B4E): one point blended: its factor the record's float at
//! +0x90 + 4 x (the source point's three sign bits, x the highest; kept in
//! `flt_93C6C8`), each of x, y, z base + (source - base) by it, rounded (`fistp`).
//!
//! In:  edi the source point, edx the base point, esi where the result goes, ebp the
//!      record (a car's +0x29C)
//! Out: ebx the sign bits, eax the source's z doubled; the rest kept
//!
//! Caller: `marks::morph_object_points`. Self-test: points of the test's (every sign
//! pattern, large and small), factors by lot; compared on the result, `flt_93C6C8` and
//! the registers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_write, rva};
use crate::math::x87;

pub const FACTOR_RVA: usize = rva(0x93C6C8); // flt_93C6C8

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn blend_corner(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let src = [get(g.edi), get(g.edi + 4), get(g.edi + 8)];
        g.ebx = src.iter().fold(0, |b, &v| (b << 1) | (v >> 31));
        g.eax = src[2].wrapping_add(src[2]);
        let f = x87::loaded(get(g.ebp + 0x90 + 4 * g.ebx));
        game_write(FACTOR_RVA, f);
        for (c, &s) in src.iter().enumerate() {
            let c = 4 * c as u32;
            let v = x87::int_lerp_to_int(s as i32, get(g.edx + c) as i32, f);
            ((g.esi + c) as *mut i32).write_unaligned(v);
        }
    }
    g
}
