//! The three floats of a vector each times the value on top of the x87 stack (x, y, z), which is
//! left as it was.
//!
//! Callers: the cars' physics under `drive::race_step` (`physics::lever_push`,
//! `physics::limit_motion`, `physics::contact_force`, `physics::bump_cars`, `physics::apply_force`,
//! and three more in unported code).

use crate::math::cells::{Cells, load};
use crate::math::x87::scale_by_80;

/// The vector at `v` times the 80-bit `factor`.
pub fn scale_vector(v: &(impl Cells + ?Sized), factor: [u8; 10]) {
    let cells = scale_by_80(load(v), factor);
    for (n, bits) in cells.into_iter().enumerate() {
        v.set_bits(n, bits);
    }
}
