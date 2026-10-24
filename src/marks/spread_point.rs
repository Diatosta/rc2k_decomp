//! `marks::spread_point` (0x4C61AB): the three values in st(1) .. st(3) each grown by
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

use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

const RANDOM_RVA: Slot = Slot::at(0x923004); // g_marksSpreadRandom
const SCALE_RVA: Slot = Slot::at(0x76521C); // dbl_76521C

/// The Rust version (see the header): the three out and eax.
pub unsafe fn spread_point(inputs: [[u8; 10]; 4]) -> ([[u8; 10]; 3], u32) {
    let [spread, b, c, d] = inputs.map(rounded);
    unsafe {
        let first = game_read::<u32>(RANDOM_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        let second = first.rotate_right(12);
        let third = second.rotate_right(12);
        game_write(RANDOM_RVA, third);
        let k: u64 = game_read(SCALE_RVA);
        let d = x87::grown(third as i32, k, d, spread);
        let c = x87::grown(second as i32, k, c, spread);
        let b = x87::grown(first as i32, k, b, spread);
        ([extended(b), extended(c), extended(d)], third)
    }
}
