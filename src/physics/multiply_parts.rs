//! `physics::multiply_parts` (0x420FD5): the three floats at ebx times those at ecx, part
//! by part, into edx (`fld; fmul; fstp`, each stored before the next is read).
//!
//! Takes: the two vectors, where the result goes (may be either)
//!
//! Caller: `sub_45F17E` (0x45F256).

use crate::math::cells::Cells;
use crate::math::x87;

/// The Rust version.
pub fn multiply_parts(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for k in 0..3 {
        out.set_bits(k, x87::mul(a.bits(k), b.bits(k)));
    }
}
