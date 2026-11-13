//! The game's 3 x 3 matrix helpers at 0x46574E .. 0x46587C (IDA), used by the
//! marks: floats, rows of 12 bytes. The three that make a turn (0x465783, 0x4657A6, 0x4657C9)
//! take their angle in st(0) and pop it.
//!
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions' results are recorded vectors in the tests (`data/vectors`).

use crate::errors::{BadInput, Refused};
use crate::math::cells::{Cells, Off};
use crate::math::x87_float::{self as bh, ONE, Rounds, X, f80, from_f80, ld, st, stf};

/// `scene::identity_matrix3` (0x46574E): the matrix `m` the identity (1.0 from `fld1`).
/// Every register kept.
pub fn identity_matrix3(m: &(impl Cells + ?Sized)) {
    for (n, &v) in rc2k_core::math::vector::IDENTITY.iter().enumerate() {
        m.set_float(n, v);
    }
}

/// `scene::multiply_matrices3` (0x4657EC): `out` = `a` x `b`, column by column, each written
/// as made (so an `out` that is one of the two behaves as there): a's row times b's
/// column, p1 + (p2 + p3) (`math::x87::row_times_column_3`).
pub fn multiply_matrices3(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    bh::at_rounding!(|r| {
        for column in 0..3 {
            for row in 0..3 {
                let v = row_times_column(r, &Off(a, 3 * row), &Off(b, column));
                out.set_float(3 * row + column, r.stf(v));
            }
        }
    });
}

/// `math::x87::row_times_column` for 3 x 3 matrices: `a`'s first three cells times `b`'s
/// cells 0, 3 and 6, p1 + (p2 + p3), before it is stored; at the rounding `r`.
fn row_times_column<R: Rounds>(
    r: R,
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
) -> R::Reg {
    let p = |k: usize| r.ld(a.float(k)) * r.ld(b.float(3 * k));
    let (p1, p2, p3) = (p(0), p(1), p(2));
    p1 + (p2 + p3)
}

/// `row_times_column` stored, as its bits (the assembly comparison's).
#[cfg(all(test, target_arch = "x86"))]
pub(crate) fn row_times_column_3(a: &(impl Cells + ?Sized), b: &(impl Cells + ?Sized)) -> u32 {
    bh::at_rounding!(|r| r.st(row_times_column(r, a, b)))
}

/// Cells holding whole numbers (a point's three dwords).
trait WholeCells: Cells {
    /// Cell `n` as the whole number it holds.
    fn whole(&self, n: usize) -> i32 {
        self.bits(n) as i32
    }
    /// Cell `n` set to the whole number `value`.
    fn set_whole(&self, n: usize, value: i32) {
        self.set_bits(n, value as u32);
    }
}

impl<C: Cells + ?Sized> WholeCells for C {}

/// A point of three whole numbers at cells `from + 3 n` turned by the matrix `m` into cells
/// `to + 3 n`: x m0 + (y m1 + z m2), x m3 + (y m4 + z m5), y m7 + (x m6 + z m8), each
/// rounded to a whole number as made (`to` may be `from`).
pub(crate) fn turn_int_point(
    from: &(impl Cells + ?Sized),
    m: &(impl Cells + ?Sized),
    to: &(impl Cells + ?Sized),
    n: usize,
) {
    bh::at_rounding!(|r| {
        let int = |k: usize| r.int(from.whole(3 * n + k));
        let (x, y, z) = (int(0), int(1), int(2));
        let c = |k: usize| r.ld(m.float(k));
        let first = x * c(0) + (y * c(1) + z * c(2));
        to.set_whole(3 * n, r.to_int(first));
        let second = x * c(3) + (y * c(4) + z * c(5));
        to.set_whole(3 * n + 1, r.to_int(second));
        let third = y * c(7) + (x * c(6) + z * c(8));
        to.set_whole(3 * n + 2, r.to_int(third));
    });
}

/// `scene::transform_int_points` (0x465829): `count` points of three whole numbers from
/// `from` turned by the matrix `m` into `to`. Refuses a count of 0x80000000 (the count would
/// run away) and one past either run's cells.
pub fn transform_int_points(
    from: &(impl Cells + ?Sized),
    to: &(impl Cells + ?Sized),
    count: u32,
    m: &(impl Cells + ?Sized),
) -> Result<(), Refused> {
    if count == 0x8000_0000 {
        return Err(BadInput("a count of 0x80000000: the original runs away").into());
    }
    let count = (count as i32).max(0) as usize;
    if 3 * count > from.count() || 3 * count > to.count() {
        return Err(BadInput("points past their run").into());
    }
    for n in 0..count {
        turn_int_point(from, m, to, n);
    }
    Ok(())
}

/// The axis a turn is about.
#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Refuses an angle `fsincos` leaves alone (finite, 2^63 or more): the original's stack
/// would then come out one short. Infinities and NaNs give NaNs, pushed as usual.
pub fn turnable(angle: &[u8; 10]) -> Result<(), Refused> {
    let exponent = u16::from_le_bytes([angle[8], angle[9]]) & 0x7FFF;
    if (0x3FFF + 63..0x7FFF).contains(&exponent) {
        return Err(BadInput("an angle fsincos cannot take").into());
    }
    Ok(())
}

/// `scene::turn_matrix3_x` (0x465783), `_y` (0x4657A6), `_z` (0x4657C9): the matrix `m` a
/// turn by the angle about that axis (the original's `fsincos` and stores, in its order).
/// Every register kept. Refuses an angle `fsincos` cannot take.
pub fn turn_matrix3(axis: Axis, m: &(impl Cells + ?Sized), angle: [u8; 10]) -> Result<(), Refused> {
    turnable(&angle)?;
    let order = match axis {
        Axis::X => TURN_X,
        Axis::Y => TURN_Y,
        Axis::Z => TURN_Z,
    };
    turn(order, angle, m);
    Ok(())
}

/// A turn's stores as cell numbers: the 1.0, the four zeros, the two cosines, the sine and
/// minus the sine.
type Stores = [usize; 9];

/// `fld tbyte; fsincos; fld1; fstp ..; fldz; fst ..` for the turn about x: the identity's
/// row and column of x, the cosine on the diagonal, the sine and minus the sine across.
const TURN_X: Stores = [0, 1, 2, 3, 6, 4, 8, 5, 7];
/// The turn about y (see `TURN_X`).
const TURN_Y: Stores = [4, 1, 3, 5, 7, 0, 8, 2, 6];
/// The turn about z (see `TURN_X`).
const TURN_Z: Stores = [8, 2, 5, 6, 7, 0, 4, 1, 3];

fn turn(stores: Stores, angle: [u8; 10], m: &(impl Cells + ?Sized)) {
    let t0 = from_f80(angle);
    let (sine, cosine) = (t0.sin(), t0.cos());
    let values = [
        ONE,
        X(0.0),
        X(0.0),
        X(0.0),
        X(0.0),
        cosine,
        cosine,
        sine,
        -sine,
    ];
    for (cell, value) in stores.into_iter().zip(values) {
        m.set_float(cell, stf(value));
    }
}

/// A float made an 80-bit value (`fld dword; fstp tbyte`).
pub fn extended(f: impl crate::math::x87_float::Loadable) -> [u8; 10] {
    f80(ld(f))
}

/// An 80-bit value rounded to a float (`fld tbyte; fstp dword`).
pub fn rounded(v: [u8; 10]) -> u32 {
    st(from_f80(v))
}

/// As `rounded`, as a float.
pub fn rounded_value(v: [u8; 10]) -> f32 {
    stf(from_f80(v))
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::Base;
    use crate::test_support::vectors::{Differential, Kind, differential, put_t80};

    /// The angle (an 80-bit value, dwords 0 .. 3), then the nine floats at dword 4. The sine
    /// and cosine are libm's: one float ulp apart is allowed.
    fn turn_test(name: &str, seed: u64, stores: Stores) {
        let test = Differential {
            name,
            cases: 6000,
            seed,
            kinds: [Kind::Ulp1; 13],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                put_t80(m, 0, rng.angle());
                m[2] &= 0xFFFF;
                m[3..].fill(0xAAAA_AAAA);
            },
            plain: &|c| turn(stores, Base::tbyte(c, 0), &c[3..12]),
        };
        differential(test);
    }

    #[test]
    fn turn_x_matches_the_assembly() {
        turn_test("scene.matrix3.turn_x", 7, TURN_X);
    }

    #[test]
    fn turn_y_matches_the_assembly() {
        turn_test("scene.matrix3.turn_y", 8, TURN_Y);
    }

    #[test]
    fn turn_z_matches_the_assembly() {
        turn_test("scene.matrix3.turn_z", 9, TURN_Z);
    }

    #[test]
    fn extended_and_rounded_matches_the_assembly() {
        // a float, its 80-bit value, the 80-bit value rounded back
        let test = Differential {
            name: "scene.matrix3.extended_and_rounded",
            cases: 8000,
            seed: 10,
            kinds: [Kind::Exact; 8],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                m[0] = rng.fl();
                put_t80(m, 16, rng.fl());
            },
            plain: &|c| {
                Base::set_tbyte(c, 4, extended(c[0].get()));
                c[7].set(rounded(Base::tbyte(c, 16)));
            },
        };
        differential(test);
    }
}
