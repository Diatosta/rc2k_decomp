//! The angle whose sine is the value on top of the x87 stack. The value is stored as a float in
//! `scratch` and held to -`one` ..= `one` (1.0); the result is the arctangent of the unheld value
//! over the root of 1 less the held float's square. A value past the limit gives +-pi/2.
//!
//! Callers: `physics::matrix_angles` (twice).

use crate::math::x87::clamped_asin_80;
use crate::session::PlainFloat;

// `scratch`
// `one`, 1.0

/// The scratch float and the limit in `scratch`, `limit` (both written, in that order).
pub fn clamped_asin_at(
    x: [u8; 10],
    scratch: &crate::session::PlainDword,
    limit: &PlainFloat,
) -> [u8; 10] {
    let mut s = scratch.get();
    let mut l = limit.get().to_bits();
    let out = clamped_asin_80(x, &mut s, &mut l);
    scratch.set(s);
    limit.set(f32::from_bits(l));
    out
}
