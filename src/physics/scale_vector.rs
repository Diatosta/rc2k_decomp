//! The three floats of a vector each times the value on top of the x87 stack (x, y, z), which is
//! left as it was.

use crate::math::cells::{Cells, load};
use crate::math::x87::scale_by_80;

/// The vector at `v` times the 80-bit `factor`.
pub(super) fn scale_vector(v: &(impl Cells + ?Sized), factor: [u8; 10]) {
    let cells = scale_by_80(load(v), factor);
    for (n, bits) in cells.into_iter().enumerate() {
        v.set_bits(n, bits);
    }
}
