//! `physics::clamped_asin` (0x42120B): the angle whose sine is st(0). The value is stored
//! as a float in `scratch`, held there to -`one` ..= `one` (1.0; the
//! limit is negated in place for the second compare and put back), and the result is
//! `fpatan` of st(0) (unheld, 80 bits) over the root of 1 less the held float's square.
//! A value past the limit gives +-pi/2.
//!
//! Callers: `physics::matrix_angles` (twice), unowned code at 0x42119E.

use crate::math::x87::clamped_asin_80;
use crate::session::Float;

// `scratch`
// `one`, 1.0

/// The scratch float and the limit in `scratch`, `limit` (both written, in that order, as the
/// original does).
pub fn clamped_asin_at(
    x: [u8; 10],
    scratch: &crate::session::PlainDword,
    limit: &Float,
) -> [u8; 10] {
    let mut s = scratch.get();
    let mut l = limit.get().to_bits();
    let out = clamped_asin_80(x, &mut s, &mut l);
    scratch.set(s);
    limit.set(f32::from_bits(l));
    out
}
