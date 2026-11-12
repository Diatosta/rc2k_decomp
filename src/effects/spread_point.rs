//! `effects::spread_point` (0x4C61AB): the three values in st(1) .. st(3) each grown by
//! itself times st(0) times a random times `MarksFloats.spread_scale` (`x87::grown`), left as st(0) ..
//! st(2).
//!
//! The random (`g_marksSpreadRandom`): times 0x9D plus 0x47, then rotated right 12 twice,
//! each of the three a whole number (signed) of it: the third for st(3), the second for
//! st(2), the first for st(1). Callers: four in the marks' particles.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::paths::Paths;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::session::PlainDword;

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
        .random
        .get()
        .wrapping_mul(0x9D)
        .wrapping_add(0x47);
    let second = first.rotate_right(12);
    let third = second.rotate_right(12);
    paths.spread_random.random.set(third);
    let k = setup.floats.spread_scale[0].get() as u64
        | (setup.floats.spread_scale[1].get() as u64) << 32;
    let d = x87::grown(third as i32, k, d, spread);
    let c = x87::grown(second as i32, k, c, spread);
    let b = x87::grown(first as i32, k, b, spread);
    ([extended(b), extended(c), extended(d)], third)
}

/// The spread point's random (IDA 0x923004 .. 0x923008).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SpreadRandom {
    /// 0x923004: `g_marksSpreadRandom`: the random (times 0x9D plus 0x47) of `effects::spread_point`.
    pub random: PlainDword,
}

impl SpreadRandom {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SpreadRandom {
            random: PlainDword::new(0x0),
        }
    }
}
