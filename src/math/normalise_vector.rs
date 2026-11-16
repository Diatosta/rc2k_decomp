//! `math::normalise_vector`: the work vector made one long.
//!
//! The vector is three floats; its length is left in `SoundSpotBlock.work_vector_length`
//! (not kept here: nothing reached reads it). The squares' sum as a float; its square root from
//! two tables (the float's bits 15 .. 22 pick one of 0x100 at `CourseLists.root_table[0]`, its
//! exponent one at `CourseLists.exponent_root_table[0]`), the two multiplied; each of the three
//! times 1 over that; a component that comes out as minus zero is made zero. A length that
//! compares equal to `Numbers.zero` (0.0), or is no number, is refused.
//!
//! Three callers.

use crate::errors::BadInput;
use crate::math::x87_float::{ONE, compare, ld, st};
use crate::memory::session::PlainDword;
use crate::race::course_lists_store::CourseLists;

// The work vector (the race's) and the root tables
// (`scene::fill_engine_tables` makes them) are the game's; the squares and the length are
// not stored, as nothing reads them.
/// `Numbers.zero`: 0.0.
const NOTHING: u32 = 0;

// ---- Rust version ----

/// The squares' sum as a float (`v0 v0 + (v1 v1 + v2 v2)` at double precision), stored
/// as a float, rounded as the control word says.
pub(crate) fn squares_sum(v: [u32; 3]) -> u32 {
    let (a, b, c) = (
        ld(v[0]) * ld(v[0]),
        ld(v[1]) * ld(v[1]),
        ld(v[2]) * ld(v[2]),
    );
    st(a + (b + c))
}

/// Each component times 1 over the length, the length being the product of the two
/// roots kept unrounded. Gives the three components as floats and the inverse as a float.
pub(crate) fn scaled_by_inverse(
    v: [u32; 3],
    root: impl crate::math::x87_float::Loadable,
    exponent_root: impl crate::math::x87_float::Loadable,
) -> ([u32; 3], u32) {
    let inverse = ONE / (ld(root) * ld(exponent_root));
    (v.map(|k| st(ld(k) * inverse)), st(inverse))
}

/// Normalises the work vector: gives the squares' sum (the bits of a float).
pub fn normalise_vector(lists: &CourseLists, work: &[PlainDword; 4]) -> Result<u32, BadInput> {
    let v: [u32; 4] = crate::memory::session::read_bits(work);
    let sum = squares_sum([v[0], v[1], v[2]]);
    let root: f32 = lists.root_table[((sum >> 15) & 0xFF) as usize].get();
    let exponent_root: f32 = lists.exponent_root_table[((sum & 0x7F80_0000) >> 23) as usize].get();
    // The product of two floats is exact in a double, as it is in the x87.
    let length = ld(root) * ld(exponent_root);
    if compare(length, ld(NOTHING)) & 0x4000 != 0 {
        return Err(BadInput("a vector of no length"));
    }
    let (scaled, inverse) = scaled_by_inverse([v[0], v[1], v[2]], root, exponent_root);
    let unsigned_zero = |bits: u32| if bits == 0x8000_0000 { 0 } else { bits };
    let _ = inverse;
    crate::memory::session::write_first(work, &scaled.map(unsigned_zero));
    Ok(sum)
}
