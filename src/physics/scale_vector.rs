//! `physics::scale_vector` (0x421297): the three floats at esi each times st(0)
//! (`fld; fmul st, st(1); fstp`, x, y, z), st(0) left as it was.
//!
//! Callers (12): the cars' physics under `drive::race_step` (0x43BDD1, `physics::lever_push`,
//! `physics::limit_motion`, `physics::contact_force`, `physics::bump_cars`, `physics::apply_force`,
//! two in unowned code at 0x43F5A0, 0x43F647).

use crate::math::cells::{Cells, load};
use crate::math::x87_run::scale_by_80;

/// The vector at `v` times the 80-bit `factor`.
pub fn scale_vector(v: &(impl Cells + ?Sized), factor: [u8; 10]) {
    let cells = scale_by_80(load(v), factor);
    for (n, bits) in cells.into_iter().enumerate() {
        v.set_bits(n, bits);
    }
}
