//! The three floats of one vector times those of another, part by part, into a third (each stored
//! before the next is read).
//!
//! The result may go into either source vector.
//!
//! Caller: `physics::contact_force`.

use crate::math::cells::Cells;
use crate::math::x87;

pub fn multiply_parts(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for k in 0..3 {
        out.set_bits(k, x87::mul(a.bits(k), b.bits(k)).to_bits());
    }
}
