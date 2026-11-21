//! `effects::jitter_point`: the three values (the second to fourth of the four) each moved
//! by the remainder of a random by the first, left as the first three.
//!
//! Three steps of the marks' random (times 0x9D plus 0x47, shifted right 12): the first moves the
//! fourth value, the second the third, the third the second.

use crate::math::matrix3::{extended, rounded};
use crate::math::x87;

/// The four values in (the spread first), the three moved ones out; also the last random >> 12.
pub fn jitter_point(
    inputs: [[u8; 10]; 4],
    effects: &crate::effects::owner::Effects,
) -> ([[u8; 10]; 3], u32) {
    let setup = &effects.setup;
    let [spread, b, c, d] = inputs.map(rounded);
    let mut moved = [d, c, b];
    let mut last = 0;
    for v in moved.iter_mut() {
        let s = setup
            .work
            .borrow()
            .seed
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        setup.work.borrow_mut().seed = s;
        last = ((s as i32) >> 12) as u32;
        *v = x87::prem_plus(last as i32, spread, *v).to_bits();
    }
    let [d, c, b] = moved;
    ([extended(b), extended(c), extended(d)], last)
}
