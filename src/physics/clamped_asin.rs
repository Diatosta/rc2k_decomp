//! The angle whose sine is the value on top of the x87 stack. The value is held to -`one` ..=
//! `one` (1.0) as a float; the result is the arctangent of the unheld value
//! over the root of 1 less the held float's square. A value past the limit gives +-pi/2.

use crate::math::x87::clamped_asin_80;
use crate::memory::session::PlainFloat;

/// The limit in `limit` (written back, as the held limit).
pub(super) fn clamped_asin_at(x: [u8; 10], limit: &PlainFloat) -> [u8; 10] {
    // The held value's float is not read again.
    let mut s = 0;
    let mut l = limit.get().to_bits();
    let out = clamped_asin_80(x, &mut s, &mut l);
    limit.set(f32::from_bits(l));
    out
}
