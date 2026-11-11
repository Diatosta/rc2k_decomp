//! Points and vectors in plain `f32` (from the x87 code of
//! `math::rotate_scratch_about_y` 0x41FEF8 and its siblings, `math::nearest_on_segment` 0x4633DE,
//! `math::normalise_vector` 0x45C3FE).
//!
//! Every step is in the original's order. What the x87 keeps on its stack is an `f64`
//! here, rounded to `f32` only where the original stores it: at the x87's double
//! precision, which the game runs at most of the time (control word 0x027F), every step rounds as an `f64` step does, so the results are the same bits;
//! at its extended precision (0x037F, after the game's `finit`) the last bit may differ
//! (`notes/conversion.md`, the float decision).

/// `fistp` under the default rounding: to the nearest, ties to even; what does not fit
/// (and NaN) is the "integer indefinite", `i32::MIN`.
pub fn fistp64(v: f64) -> i32 {
    let r = v.round_ties_even();
    if r.is_nan() || r < i32::MIN as f64 || r >= 2147483648.0 {
        i32::MIN
    } else {
        r as i32
    }
}

/// `fistp` of a float loaded from memory.
pub fn fistp(v: f32) -> i32 {
    fistp64(v as f64)
}

/// The sine and cosine tables' entry of an angle (a dword's turn is 2^32; 4096 entries).
pub fn turn_entry(angle: u32) -> u32 {
    angle.wrapping_add(0x8_0000) >> 20
}

/// x and z turned: x' = x cos + z sin, z' = z cos - x sin.
pub fn turn_xz(x: f32, z: f32, sine: f32, cosine: f32) -> (f32, f32) {
    let (x, z, s, c) = (x as f64, z as f64, sine as f64, cosine as f64);
    ((x * c + z * s) as f32, (z * c - x * s) as f32)
}

/// `math::nearest_on_segment`'s block (0x71C384, 21 floats): the ends A and B, the point
/// O, the result, t, the work and the zero it is compared with.
pub mod nearest {
    pub const A: usize = 0;
    pub const B: usize = 3;
    pub const O: usize = 6;
    pub const RESULT: usize = 9;
    pub const T: usize = 12;
    /// A - O (x, z), B - O (x, z).
    pub const AO: usize = 13;
    pub const BO: usize = 15;
    /// What the denominator is compared with (0.0 in the game).
    pub const ZERO: usize = 17;
    /// A - B (x, z), from the two differences.
    pub const AB: usize = 19;
    pub const LEN: usize = 21;
}

/// The point of segment AB nearest O, seen from above (x and z only): t = (A - O).(A - B)
/// over |A - B|^2, 0 when that is equal to the block's zero (or unordered); held to `low`
/// .. `high` (0 and 1 in the game); the point A + (B - A) t in all three.
pub fn nearest_on_segment(block: &mut [f32; nearest::LEN], low: f32, high: f32) {
    use nearest::*;
    let at = |block: &[f32; LEN], n: usize| block[n] as f64;
    // Each difference stored (`fstp`).
    block[AO] = (at(block, A) - at(block, O)) as f32;
    block[AO + 1] = (at(block, A + 2) - at(block, O + 2)) as f32;
    block[BO] = (at(block, B) - at(block, O)) as f32;
    block[BO + 1] = (at(block, B + 2) - at(block, O + 2)) as f32;
    block[AB] = (at(block, AO) - at(block, BO)) as f32;
    block[AB + 1] = (at(block, AO + 1) - at(block, BO + 1)) as f32;
    // These three stay on the stack.
    let numerator = at(block, AO) * at(block, AB) + at(block, AO + 1) * at(block, AB + 1);
    let other = at(block, BO) * at(block, AB) + at(block, BO + 1) * at(block, AB + 1);
    let denominator = numerator - other;
    let zero = at(block, ZERO);
    // `fcom`: C3 for equal or unordered.
    if denominator == zero || denominator.is_nan() || zero.is_nan() {
        block[T] = f32::from_bits(0);
    } else {
        let mut t = (numerator / denominator) as f32;
        // Raised to low unless at or above it; lowered to high when at or above it.
        if t.partial_cmp(&low).is_none_or(std::cmp::Ordering::is_lt) {
            t = low;
        }
        if t >= high {
            t = high;
        }
        block[T] = t;
    }
    let t = at(block, T);
    for n in 0..3 {
        block[RESULT + n] = ((at(block, B + n) - at(block, A + n)) * t + at(block, A + n)) as f32;
    }
}

/// What normalising the x87 way leaves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normalised {
    /// The squares' sum, as the original leaves its bits in eax.
    pub sum: f32,
    pub length: f32,
    /// The vector, a component that comes out minus zero made plus zero.
    pub vector: [f32; 3],
    /// 1 over the length.
    pub reciprocal: f32,
}

/// The vector made one long, the x87 way: the squares' sum; its square root from two
/// tables (the float's bits 15 .. 22 pick one of 256 in `roots`, its exponent one of
/// `exponent_roots`), the two multiplied; each component times 1 over that. `None` for a
/// length equal to `nothing` (or unordered): the original ends there with the length on
/// the x87's stack.
pub fn normalise(
    v: [f32; 3],
    roots: &[f32; 256],
    exponent_roots: &[f32; 256],
    nothing: f32,
) -> Option<Normalised> {
    let w = v.map(|x| x as f64);
    // The sum stored (`fstp`), its bits picking the roots.
    let sum = (w[0] * w[0] + (w[1] * w[1] + w[2] * w[2])) as f32;
    let bits = sum.to_bits();
    let root = roots[((bits >> 15) & 0xFF) as usize] as f64;
    let exponent_root = exponent_roots[((bits & 0x7F80_0000) >> 23) as usize] as f64;
    // The length stays on the stack (`fst` stores a rounded copy).
    let length = root * exponent_root;
    let nothing = nothing as f64;
    if length == nothing || length.is_nan() || nothing.is_nan() {
        return None;
    }
    let reciprocal = 1.0 / length;
    let plus_zero = |x: f32| if x.to_bits() == 0x8000_0000 { 0.0 } else { x };
    Some(Normalised {
        sum,
        length: length as f32,
        vector: w.map(|x| plus_zero((x * reciprocal) as f32)),
        reciprocal: reciprocal as f32,
    })
}

/// What normalising the SSE way leaves: the four squares, the length and the four
/// components (the fourth too) times 1 over the length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalisedFour {
    pub squares: [f32; 4],
    pub length: f32,
    pub vector: [f32; 4],
}

/// The SSE way. The original's `rsqrtss` and `rcpss` are approximations that differ
/// between processors; here they are exact (a decided difference: within 1 part in 2^11).
pub fn normalise_four(v: [f32; 4]) -> NormalisedFour {
    let squares = v.map(|x| x * x);
    let sum = squares[0] + squares[1] + squares[2];
    let reciprocal_root = 1.0 / sum.sqrt();
    NormalisedFour {
        squares,
        length: 1.0 / reciprocal_root,
        vector: v.map(|x| reciprocal_root * x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_and_turns() {
        assert_eq!(fistp(2.5), 2);
        assert_eq!(fistp(3.5), 4);
        assert_eq!(fistp(-2.5), -2);
        assert_eq!(fistp(3e9), i32::MIN);
        assert_eq!(fistp(f32::NAN), i32::MIN);
        assert_eq!(turn_entry(0x4000_0000), 0x400);
        let (x, z) = turn_xz(1.0, 0.0, 1.0, 0.0);
        assert_eq!((x, z), (0.0, -1.0));
    }

    #[test]
    fn nearest_points() {
        let mut b = [0.0f32; nearest::LEN];
        b[nearest::B..nearest::B + 3].copy_from_slice(&[10.0, 2.0, 0.0]);
        b[nearest::O..nearest::O + 3].copy_from_slice(&[5.0, 0.0, 3.0]);
        nearest_on_segment(&mut b, 0.0, 1.0);
        assert_eq!(b[nearest::T], 0.5);
        assert_eq!(&b[nearest::RESULT..nearest::RESULT + 3], &[5.0, 1.0, 0.0]);
        // Past B: held to 1.
        b[nearest::O] = 20.0;
        nearest_on_segment(&mut b, 0.0, 1.0);
        assert_eq!(b[nearest::T], 1.0);
        // A segment of no length.
        b[nearest::B..nearest::B + 3].copy_from_slice(&[0.0, 7.0, 0.0]);
        nearest_on_segment(&mut b, 0.0, 1.0);
        assert_eq!(b[nearest::T], 0.0);
    }

    #[test]
    fn normalising() {
        let roots = [2.0f32; 256];
        let exponent_roots = [2.5f32; 256];
        let n = normalise([3.0, 4.0, -0.0], &roots, &exponent_roots, 0.0).unwrap();
        assert_eq!((n.sum, n.length), (25.0, 5.0));
        assert_eq!(n.vector, [0.6, 0.8, 0.0]);
        assert_eq!(n.vector[2].to_bits(), 0);
        assert!(normalise([1.0, 0.0, 0.0], &[0.0; 256], &exponent_roots, 0.0).is_none());
        let f = normalise_four([3.0, 4.0, 0.0, 1.0]);
        assert_eq!((f.length, f.vector[0]), (5.0, 0.6));
    }
}
