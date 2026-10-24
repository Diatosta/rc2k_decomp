//! `scene::light_dot` (0x44EC18): how much a light falls on a point, as a whole number.
//!
//! In:  nothing
//! Out: eax = the number; everything else preserved
//!
//! The three floats at `dword_62DAA8` times the three at `dword_62DAB4`, added (the second and
//! third products first), times `flt_62DAC0`, rounded as the x87 is set to (`fistp`); also left
//! in `dword_62400C`. Callers: the two lighting functions `sub_4511F1` and `sub_451296`, which
//! put a point's place in the first three, a light's direction in the second and its strength
//! in the factor.

use crate::framework::{game_read, game_write, rva};
use crate::scene::draw_polygon::CHECKED_RVA;

// ---- game data ----

/// `dword_62DAA8` .. `dword_62DAB0` the point, `dword_62DAB4` .. `dword_62DABC` the
/// light, `flt_62DAC0` the factor.
pub const INPUTS_RVA: usize = rva(0x62DAA8);

// ---- Rust version ----

pub unsafe fn light_dot() -> i32 {
    unsafe {
        let v: [u32; 7] = game_read(INPUTS_RVA);
        let mut out = 0i32;
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fmul dword ptr [{v} + 12]",
            "fld dword ptr [{v} + 4]",
            "fmul dword ptr [{v} + 16]",
            "fld dword ptr [{v} + 8]",
            "fmul dword ptr [{v} + 20]",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fmul dword ptr [{v} + 24]",
            "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        game_write(CHECKED_RVA, out);
        out
    }
}
