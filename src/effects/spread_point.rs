//! `effects::spread_point` (0x4C61AB): the three values in st(1) .. st(3) each grown by
//! itself times st(0) times a random times `dbl_76521C` (`x87::grown`), left as st(0) ..
//! st(2).
//!
//! In:  st(0) the spread, st(1) .. st(3) the point (each rounded to a float)
//! Out: st(0) .. st(2) the point grown (st(1)'s first); eax the random as left; the rest
//!      kept
//!
//! The random (`g_marksSpreadRandom`): times 0x9D plus 0x47, then rotated right 12 twice,
//! each of the three a whole number (signed) of it: the third for st(3), the second for
//! st(2), the first for st(1). Callers: four in the marks' particles.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::paths::Paths;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::session::Dword;

/// The Rust version (see the header): the three out and eax.
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
    let k = setup.floats.spread_scale.get_u32(0) as u64
        | (setup.floats.spread_scale.get_u32(4) as u64) << 32;
    let d = x87::grown(third as i32, k, d, spread);
    let c = x87::grown(second as i32, k, c, spread);
    let b = x87::grown(first as i32, k, b, spread);
    ([extended(b), extended(c), extended(d)], third)
}

/// The spread point's random (IDA 0x923004 .. 0x923008, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SpreadRandom {
    /// 0x923004: `g_marksSpreadRandom`: the random (times 0x9D plus 0x47) of `effects::spread_point`.
    pub random: Dword,
}

impl SpreadRandom {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SpreadRandom {
            random: Dword::new(0x0),
        }
    }
    pub const START: usize = 0x923004;
    pub const LEN: usize = 0x4;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::effects::owner::EFFECTS.paths.spread_random)
            as usize
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn spread_random_starts_as_the_games() {
        assert_eq!(size_of::<SpreadRandom>(), SpreadRandom::LEN);
        let image = crate::moved::image_bytes(SpreadRandom::START, SpreadRandom::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(SpreadRandom::home()).cast_const(),
                SpreadRandom::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SPREAD_RANDOM starts differently from the image"
        );
    }
}
