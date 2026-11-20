//! `effects::spread_point`: three values each grown by itself times a factor (the first of the four
//! inputs) times a random times `MarksFloats.spread_scale` (`x87::grown`).
//!
//! The random: times 0x9D plus 0x47 (the marks' seed), then rotated right 12 twice, each of the
//! three a whole number (signed) of it: the third for the last value, the second for the middle
//! one, the first for the first.

use crate::effects::paths_store::Paths;
use crate::math::matrix3::{extended, rounded};
use crate::math::x87;

/// The three results.
pub fn spread_point(
    paths: &Paths,
    inputs: [[u8; 10]; 4],
    effects: &crate::effects::owner::Effects,
) -> ([[u8; 10]; 3], u32) {
    let setup = &effects.setup;
    let [spread, b, c, d] = inputs.map(rounded);

    let first = paths
        .spread_random
        .get()
        .wrapping_mul(0x9D)
        .wrapping_add(0x47);
    let second = first.rotate_right(12);
    let third = second.rotate_right(12);
    paths.spread_random.set(third);
    let k = setup.floats.spread_scale[0].get() as u64
        | (setup.floats.spread_scale[1].get() as u64) << 32;
    let d = x87::grown(third as i32, k, d, spread);
    let c = x87::grown(second as i32, k, c, spread);
    let b = x87::grown(first as i32, k, b, spread);
    ([extended(b), extended(c), extended(d)], third)
}
