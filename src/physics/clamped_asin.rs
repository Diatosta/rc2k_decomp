//! `physics::clamped_asin` (0x42120B): the angle whose sine is st(0). The value is stored
//! as a float in `dword_62400C`, held there to -`flt_5C5D38` ..= `flt_5C5D38` (1.0; the
//! limit is negated in place for the second compare and put back), and the result is
//! `fpatan` of st(0) (unheld, 80 bits) over the root of 1 less the held float's square.
//! A value past the limit gives +-pi/2.
//!
//! In:  st(0) the sine
//! Out: st(0) the angle (in its place); every register kept; flags changed (no caller
//!      reads them)
//! Callers: `physics::matrix_angles` (twice), unowned code at 0x42119E.

use crate::math::x87_run::clamped_asin_80;
use crate::session::Cell32;

// dword_62400C
// flt_5C5D38, 1.0

/// The Rust core: the scratch float and the limit in `scratch`, `limit` (both written, in
/// that order, as the original does).
pub fn clamped_asin_at(x: [u8; 10], scratch: &dyn Cell32, limit: &dyn Cell32) -> [u8; 10] {
    let mut s = scratch.bits();
    let mut l = limit.bits();
    let out = clamped_asin_80(x, &raw mut s, &raw mut l);
    scratch.set_bits(s);
    limit.set_bits(l);
    out
}
