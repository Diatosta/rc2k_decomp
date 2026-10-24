//! `marks::blend_points` (0x4BA5E2): three points of whole numbers (eax, ebx, ecx) blended
//! by the weights of entry `dl` of the spline file (`g_marksSplineFile`, 12 bytes each:
//! c's weight, then b's, then a's): each of x, y, z a ka + (b kb + c kc), left unrounded in
//! st(2) .. st(0) (x deepest).
//!
//! In:  `a`, `b`, `c` the points, `entry` the entry
//! Out: x, y, z (the original's st(2) .. st(0))
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

use crate::marks::splines::FILE_RVA;
use crate::marks::store::game_read;
use crate::math::x87;

/// The Rust version (see the header): the three values (x, y, z).
pub unsafe fn blend_points(a: u32, b: u32, c: u32, entry: u8) -> [[u8; 10]; 3] {
    let at = FILE_RVA + 12 * entry as usize;
    let k: [u32; 3] = unsafe { [game_read(at + 8), game_read(at + 4), game_read(at)] };
    let p = |r: u32, c: u32| unsafe { ((r + 4 * c) as *const i32).read_unaligned() };
    std::array::from_fn(|n| {
        let n = n as u32;
        x87::blend3_80(p(a, n), p(b, n), p(c, n), k)
    })
}
