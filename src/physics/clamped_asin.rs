//! The angle whose sine is the value on top of the x87 stack. The value is held to -`one` ..=
//! `one` (1.0) as a float; the result is the arctangent of the unheld value
//! over the root of 1 less the held float's square. A value past the limit gives +-pi/2.

use crate::math::x87::clamped_asin_80;
use crate::memory::session::ConstFloat;

/// The sine's angle with the limit `limit`. The limit the helper holds the value to is its own
/// copy: it ends as it began, so the constant is not written back.
pub(super) fn clamped_asin_at(x: [u8; 10], limit: &ConstFloat) -> [u8; 10] {
    // The held value's float is not read again.
    let mut s = 0;
    let mut l = limit.bits();
    clamped_asin_80(x, &mut s, &mut l)
}
