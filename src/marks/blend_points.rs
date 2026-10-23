//! `marks::blend_points` (0x4BA5E2): three points of whole numbers (eax, ebx, ecx) blended
//! by the weights of entry `dl` of the spline file (`g_marksSplineFile`, 12 bytes each:
//! c's weight, then b's, then a's): each of x, y, z a ka + (b kb + c kc), left unrounded in
//! st(2) .. st(0) (x deepest).
//!
//! In:  eax, ebx, ecx the points, dl the entry
//! Out: st(0) z, st(1) y, st(2) x; edx the entry times 12; the rest kept
//!
//! Thunk by `out3_hook!`. Caller: `sub_4BA646`. Self-test: points of the test's and
//! entries by lot (weights the game's file's, or put by the test when the file is empty),
//! compared on the three values and the registers.
//!
//! Converted 2026-10-21: the hook's Rust version is `blend_points_plain`, through
//! `rc2k_core::marks::particles::blend` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read};
use crate::marks::splines::FILE_RVA;
use crate::math::x87;

/// The Rust version (see the header): the three values (x, y, z) and the registers.
pub unsafe fn blend_points(entry: &Regs) -> ([[u8; 10]; 3], Regs) {
    let mut g = *entry;
    g.edx = (g.edx & 0xFF) * 12;
    let at = FILE_RVA + g.edx as usize;
    let k: [u32; 3] = unsafe { [game_read(at + 8), game_read(at + 4), game_read(at)] };
    let p = |r: u32, c: u32| unsafe { ((r + 4 * c) as *const i32).read_unaligned() };
    let v = std::array::from_fn(|c| {
        let c = c as u32;
        x87::blend3_80(p(g.eax, c), p(g.ebx, c), p(g.ecx, c), k)
    });
    (v, g)
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::particles::blend`
pub unsafe fn blend_points_plain(entry: &Regs) -> ([[u8; 10]; 3], Regs) {
    let mut g = *entry;
    g.edx = (g.edx & 0xFF) * 12;
    let at = FILE_RVA + g.edx as usize;
    let k: [u32; 3] = unsafe { game_read(at) };
    let p = |r: u32| -> [i32; 3] {
        std::array::from_fn(|c| unsafe { ((r + 4 * c as u32) as *const i32).read_unaligned() })
    };
    let v =
        rc2k_core::marks::particles::blend([p(g.eax), p(g.ebx), p(g.ecx)], k.map(f32::from_bits));
    (v.map(rc2k_core::math::extended::from_f64), g)
}
