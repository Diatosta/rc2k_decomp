//! `scene::rotate_point` (0x44D110): the point in st(0) .. st(2) turned and placed by the
//! matrix (rows 0x10 bytes apart, the place at +0x30), left there unrounded.
//!
//! Plain Rust: the x87's arithmetic in `X` (`math/x87_float.rs`), the point
//! and the result as 80-bit values (exact for what `X` holds). The original's instructions
//! are recorded in the tests' `reference_vectors`. Refuses a null matrix.

use crate::math::cells::{Cells, load};
use crate::math::x87_float::{Rounds, at_rounding};
use crate::scene::x87_mem::{Base, fld_with, fld80_with, fstp80_with};

/// The Rust version: `p` as st(0) .. st(2), the result the same way.
pub fn rotate_point(m: &(impl Cells + ?Sized), p: [[u8; 10]; 3]) -> [[u8; 10]; 3] {
    let mut out = [[0u8; 10]; 3];
    let m: [u32; 15] = load(m);
    rotate(
        &p[..],
        &m[..],
        std::cell::Cell::from_mut(&mut out[..]).as_slice_of_cells(),
    );
    out
}

/// `fld tbyte` x3, the three rows' sums of products plus the place, `fstp tbyte` x3.
fn rotate(p: impl Base, m: impl Base, o: impl Base) {
    at_rounding!(|r| {
        let point = [0x0, 0xA, 0x14].map(|off| fld80_with(r, p, off));
        let [t12, t22, t32] = turned(r, m, point);
        fstp80_with(r, o, 0x0, t12);
        fstp80_with(r, o, 0xA, t22);
        fstp80_with(r, o, 0x14, t32);
    });
}

/// The point's three rows' sums of products plus the place, each with the x87's order, at
/// the rounding `r`.
pub(crate) fn turned<R: Rounds>(r: R, m: impl Base, [t2, t1, t0]: [R::Reg; 3]) -> [R::Reg; 3] {
    let fld = |m, off| fld_with(r, m, off);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x30);
    let t12 = t10 + t11;
    let t13 = fld(m, 0x4);
    let t14 = t13 * t2;
    let t15 = fld(m, 0x14);
    let t16 = t15 * t1;
    let t17 = fld(m, 0x24);
    let t18 = t17 * t0;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = fld(m, 0x34);
    let t22 = t20 + t21;
    let t23 = fld(m, 0x8);
    let t24 = t2 * t23;
    let t25 = fld(m, 0x18);
    let t26 = t1 * t25;
    let t27 = fld(m, 0x28);
    let t28 = t0 * t27;
    let t29 = t28 + t26;
    let t30 = t24 + t29;
    let t31 = fld(m, 0x38);
    let t32 = t30 + t31;
    [t12, t22, t32]
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential, put_t80};

    #[test]
    fn rotate_matches_the_assembly() {
        // bytes 0 .. 30 the point, 32 .. 92 the matrix (15 dwords), 96 .. 126 the result
        let test = Differential {
            name: "scene.rotate_point.rotate",
            cases: 6000,
            seed: 2,
            kinds: [Kind::Exact; 32],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                for k in 0..3 {
                    put_t80(m, 10 * k, rng.fl());
                }
                for d in m.iter_mut().skip(8).take(15) {
                    *d = rng.fl();
                }
            },
            plain: &|c| rotate(c, &c[8..], &c[24..]),
        };
        differential(test);
    }
}
