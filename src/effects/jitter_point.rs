//! `effects::jitter_point` (0x4C6106): the three values in st(1) .. st(3) each moved by the
//! remainder of a random by st(0) (`fprem`), left as st(0) .. st(2).
//!
//! Three steps of the marks' random (times 0x9D plus 0x47, shifted right 12): the first
//! moves st(3), the second st(2), the third st(1). Nine callers in the marks' particles.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

/// The four as st(0) .. st(3), the three out the same
/// way; the last random >> 12.
pub fn jitter_point(
    inputs: [[u8; 10]; 4],
    effects: &crate::effects::owner::Effects,
) -> ([[u8; 10]; 3], u32) {
    let setup = &effects.setup;
    let [spread, b, c, d] = inputs.map(rounded);
    let mut moved = [d, c, b];
    let mut last = 0;
    for v in moved.iter_mut() {
        let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        setup.work.seed.set(s);
        last = ((s as i32) >> 12) as u32;
        *v = x87::prem_plus(last as i32, spread, *v);
    }
    let [d, c, b] = moved;
    ([extended(b), extended(c), extended(d)], last)
}
