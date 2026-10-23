//! `marks::clamp_pair` (0x4BEF22): the pair in st(0), st(1) scaled to whole numbers, held
//! to 0 .. 0x7FFF and 0 .. 0xFFFF, and scaled back.
//!
//! In:  st(0), st(1) the pair
//! Out: ebx, eax the whole numbers; st(0), st(1) them scaled back (unrounded)
//!
//! See `marks/turns.rs`. Thunk by `pair_hook!`. Callers: `sub_4BEB25` (twice). Self-test:
//! the original and the Rust version on pairs by lot (negative, large, NaN among them),
//! at the test precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::turns::*;

/// The converted `clamp_pair` (plain floats): the hook's. The x87 version stays for
/// `marks::make_corners` (unconverted).
// Converted: `rc2k_core::marks::turns::clamp_pair`
pub unsafe fn clamp_pair_plain(entry: &Regs, inputs: [[u8; 10]; 2]) -> (Regs, [[u8; 10]; 2]) {
    use rc2k_core::math::extended::{from_f64, to_f64};
    let mut g = *entry;
    let k: [u32; 4] = unsafe { crate::framework::game_read(PAIR_SCALES_RVA) };
    let (b, a, out) = rc2k_core::marks::turns::clamp_pair(
        to_f64(inputs[0]),
        to_f64(inputs[1]),
        k.map(f32::from_bits),
    );
    [g.ebx, g.eax] = [b, a];
    (g, out.map(from_f64))
}
