//! `marks::scale_vectors_by` (0x4D16A3): two vectors scaled: `flt_764FA0` .. =
//! `dword_764F94` .. by `dword_764F90`, `flt_764FBC` .. = `dword_764FB0` .. by
//! `dword_764FAC` (`fld; fmul`, three on the stack, stored last first).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4D14B4`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

/// (the factor, the vector, the result).
const PAIRS: [(Slot, Slot, Slot); 2] = [
    (Slot::at(0x764F90), Slot::at(0x764F94), Slot::at(0x764FA0)),
    (Slot::at(0x764FAC), Slot::at(0x764FB0), Slot::at(0x764FBC)),
];

/// The Rust version (see the header).
pub unsafe fn scale_vectors_by() {
    unsafe { scale_triples(&PAIRS) }
}

/// Each (factor, vector, result): the vector's three floats by the factor (`fld; fmul`).
pub unsafe fn scale_triples(pairs: &[(Slot, Slot, Slot)]) {
    unsafe {
        for &(factor, from, to) in pairs {
            let k: u32 = game_read(factor);
            for n in 0..3 {
                game_write(to + 4 * n, x87::mul(game_read(from + 4 * n), k));
            }
        }
    }
}
