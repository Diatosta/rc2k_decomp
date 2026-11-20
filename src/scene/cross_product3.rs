//! `scene::cross_product3`: the cross product of the two vectors of three floats into a third:
//! x = a.y b.z - a.z b.y, y = a.z b.x - a.x b.z, z = a.x b.y - a.y b.x, each rounded as it is
//! written (so an output over an input is read back as it then is).
//!
//! See `scene/vector3.rs`. 16 callers (the marks' tyre marks and their kin). Plain Rust: the
//! arithmetic is the x87's, in `X` (`math/x87_float.rs`); the tests compare against recorded
//! vectors (`data/vectors`).

use crate::math::cells::Cells;
use crate::math::x87_float::{ld, stf};

/// The three products-differences, each stored as made: an output over an input is read back
/// as it then is.
pub fn cross(a: &(impl Cells + ?Sized), b: &(impl Cells + ?Sized), o: &(impl Cells + ?Sized)) {
    let (a, b) = (|k| ld(a.float(k)), |k| ld(b.float(k)));
    o.set_float(0, stf(a(1) * b(2) - a(2) * b(1)));
    o.set_float(1, stf(a(2) * b(0) - a(0) * b(2)));
    o.set_float(2, stf(a(0) * b(1) - a(1) * b(0)));
}

/// `cross` of two plain vectors into a third, the same steps (the output is none of the inputs).
pub fn cross_plain(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    let (a, b) = (|k: usize| ld(a[k]), |k: usize| ld(b[k]));
    [
        stf(a(1) * b(2) - a(2) * b(1)),
        stf(a(2) * b(0) - a(0) * b(2)),
        stf(a(0) * b(1) - a(1) * b(0)),
    ]
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, Rng, differential};
    use std::cell::Cell;

    /// dwords 0..3 a, 3..6 b, 6..9 the result, 9 the mode
    fn fill(rng: &mut Rng, m: &mut [u32; 10]) {
        for d in m.iter_mut().take(9) {
            *d = rng.fl();
        }
        m[9] = rng.below(3) as u32;
    }

    /// 0: apart; 1: the result is a; 2: the result is b
    fn result_at(mode: u32) -> usize {
        match mode {
            1 => 0,
            2 => 3,
            _ => 6,
        }
    }

    fn plain(m: &[Cell<u32>]) {
        let o = result_at(m[9].get());
        cross(&m[0..3], &m[3..6], &m[o..o + 3]);
    }

    fn test() -> Differential<'static, 10> {
        Differential {
            name: "scene.cross_product3",
            cases: 5000,
            seed: 1,
            kinds: [Kind::Float; 10],
            fill: &fill,
            plain: &plain,
        }
    }

    /// Distinct vectors, then the result over an input.
    #[test]
    fn cross_matches_the_assembly() {
        differential(test());
    }
}
