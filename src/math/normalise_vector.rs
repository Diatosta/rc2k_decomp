//! `math::normalise_vector`: the work vector made one long.
//!
//! The vector is three floats; its length is left in `SoundSpotBlock.work_vector_length`
//! (not kept here: nothing reached reads it). Two ways, by the switch
//! `DrawSwitches.use_sse` (a key's toggle and the processor's SSE together; 0 as the game
//! starts, so the x87 way is the one in use):
//! - x87: the squares' sum as a float; its square root from two tables (the float's
//!   bits 15 .. 22 pick one of 0x100 at `CourseLists.root_table[0]`, its exponent one at
//!   `CourseLists.exponent_root_table[0]`), the two multiplied; each of the three times 1
//!   over that; a component that comes out as minus zero is made zero. A length that
//!   compares equal to `Numbers.zero` (0.0), or is no number, is refused.
//! - SSE: the four floats squared, the first three added, `rsqrtss`, `rcpss` of that to
//!   `SoundSpotBlock.work_vector_length`, and all four (the fourth too) times the
//!   `rsqrtss`. Both instructions are approximations that differ between processors: one
//!   processor's results, as tables (`arch::rsqrtss`), on every machine.
//!
//! Three callers.

use crate::errors::BadInput;
use crate::math::x87_float::{ONE, compare, ld, st};
use crate::race::course_lists_store::CourseLists;
use crate::scene::draw_switches_store::DrawSwitches;
use crate::session::PlainDword;

// The work vector and the SSE switch (the race's) and the root tables
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

/// Normalises the work vector. The x87 way gives `Some` of the squares' sum (the bits of a
/// float); the SSE way gives `None`.
pub fn normalise_vector(
    lists: &CourseLists,
    switches: &DrawSwitches,
    work: &[PlainDword; 4],
) -> Result<Option<u32>, BadInput> {
    normalise(lists, work, switches.use_sse.is_set())
}

/// The SSE path: each component times the approximate reciprocal root of the squares'
/// sum, in single precision. The fourth component is scaled too.
fn sse_normalise(v: [u32; 4]) -> [u32; 4] {
    let v = v.map(f32::from_bits);
    let sum = (v[0] * v[0] + v[1] * v[1]) + v[2] * v[2];
    let root = crate::arch::rsqrtss(sum);
    v.map(|k| (root * k).to_bits())
}

/// The x87 at its precision and the SSE instructions: what the callers in modules not yet
/// converted use.
pub fn normalise(
    lists: &CourseLists,
    work: &[PlainDword; 4],
    sse: bool,
) -> Result<Option<u32>, BadInput> {
    let v: [u32; 4] = crate::session::read_bits(work);
    if sse {
        crate::session::write_bits(work, sse_normalise(v));
        return Ok(None);
    }
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
    crate::session::write_first(work, &scaled.map(unsigned_zero));
    Ok(Some(sum))
}

#[cfg(test)]
mod tests {
    use crate::test_support::frozen::{Canon, Digest, check_one};

    /// The SSE path (`sse_normalise`: `mulps`, `rsqrtss`, `rcpss` from the tables of
    /// `arch`) over seeded vectors, against a frozen hash of the results.
    #[test]
    fn the_sse_path_as_the_instructions_gave() {
        let mut x = 0x9E37_79B9u32;
        let mut digest = Digest::default();
        for _ in 0..200_000 {
            let mut next = || {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x
            };
            let scale = |r: u32| (r as i32 as f32 / 2.0f32.powi(31) * 500.0).to_bits();
            let v = [scale(next()), scale(next()), scale(next()), next()];
            super::sse_normalise(v).feed(&mut digest);
        }
        check_one("normalise_vector", "sse_normalise", digest, FROZEN);
    }

    /// The results' hash (`RC2K_FREEZE_OUT` makes it again).
    const FROZEN: &[(&str, u64)] = &[("normalise_vector::sse_normalise", 0x9e32f6b98da9268e)];
}
