//! `marks::jitter_point` (0x4C6106): the three values in st(1) .. st(3) each moved by the
//! remainder of a random by st(0) (`fprem`), left as st(0) .. st(2).
//!
//! In:  st(0) the spread, st(1) .. st(3) the point (each rounded to a float)
//! Out: st(0) .. st(2) the point moved (st(1)'s first); ebx the last random >> 12
//!      (arithmetic); the rest kept
//!
//! Three steps of the marks' random (times 0x9D plus 0x47, shifted right 12): the first
//! moves st(3), the second st(2), the third st(1). Thunk by `quad_triple_hook!`. Nine callers
//! in the marks' particles. Self-test: the original and the Rust version on spreads,
//! points and randoms by lot; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `jitter_point_plain`, through
//! `rc2k_core::marks::particles::jitter` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write};
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

/// The Rust version (see the header): the four as st(0) .. st(3), the three out the same
/// way; the last random >> 12.
pub unsafe fn jitter_point(inputs: [[u8; 10]; 4]) -> ([[u8; 10]; 3], u32) {
    let [spread, b, c, d] = inputs.map(rounded);
    let mut moved = [d, c, b];
    let mut last = 0;
    unsafe {
        for v in moved.iter_mut() {
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            last = ((s as i32) >> 12) as u32;
            *v = x87::prem_plus(last as i32, spread, *v);
        }
    }
    let [d, c, b] = moved;
    ([extended(b), extended(c), extended(d)], last)
}
