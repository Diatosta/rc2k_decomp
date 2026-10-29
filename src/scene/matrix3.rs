//! The game's 3 x 3 matrix helpers at 0x46574E .. 0x46587C (IDA, 2026-10-16), used by the
//! marks: floats, rows of 12 bytes. The three that make a turn (0x465783, 0x4657A6, 0x4657C9)
//! take their angle in st(0) and pop it.
//!
//! Plain Rust since 2026-10-24 (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions are `asm_reference` in the tests.

use crate::errors::{BadInput, Refused};
use crate::math::cells::{Cells, Off};
use crate::math::x87_float::{self as bh, ONE, X, f80, from_f80, ld, st};

/// `scene::identity_matrix3` (0x46574E): the matrix `m` the identity (1.0 from `fld1`).
/// Every register kept.
pub fn identity_matrix3(m: &(impl Cells + ?Sized)) {
    for (n, v) in rc2k_core::math::vector::IDENTITY.iter().enumerate() {
        m.set_bits(n, v.to_bits());
    }
}

/// `scene::multiply_matrices3` (0x4657EC): `out` = `a` x `b`, column by column, each written
/// as made (so an `out` that is one of the two behaves as there): a's row times b's
/// column, p1 + (p2 + p3) (`math::x87::row_times_column_3`). The registers the original
/// leaves (ecx and edx 0, edi and ebp 12 on) are read by no caller (liveness at the five
/// call sites, all the effects', which are Rust).
pub fn multiply_matrices3(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for column in 0..3 {
        for row in 0..3 {
            let v = row_times_column_3(&Off(a, 3 * row), &Off(b, column));
            out.set_bits(3 * row + column, v);
        }
    }
}

/// `math::x87::row_times_column` for 3 x 3 matrices: `a`'s first three cells times `b`'s
/// cells 0, 3 and 6, p1 + (p2 + p3).
pub(crate) fn row_times_column_3(a: &(impl Cells + ?Sized), b: &(impl Cells + ?Sized)) -> u32 {
    let p = |k: usize| ld(a.bits(k)) * ld(b.bits(3 * k));
    let (p1, p2, p3) = (p(0), p(1), p(2));
    st(p1 + (p2 + p3))
}

/// A point of three whole numbers at cells `from + 3 n` turned by the matrix `m` into cells
/// `to + 3 n`: x m0 + (y m1 + z m2), x m3 + (y m4 + z m5), y m7 + (x m6 + z m8), each
/// rounded to a whole number as made (`to` may be `from`).
pub(crate) fn turn_int_point(
    from: &(impl Cells + ?Sized),
    m: &(impl Cells + ?Sized),
    to: &(impl Cells + ?Sized),
    n: usize,
) {
    let int = |k: usize| bh::int(from.bits(3 * n + k) as i32);
    let (x, y, z) = (int(0), int(1), int(2));
    let c = |k: usize| ld(m.bits(k));
    let first = x * c(0) + (y * c(1) + z * c(2));
    to.set_bits(3 * n, bh::to_int(first) as u32);
    let second = x * c(3) + (y * c(4) + z * c(5));
    to.set_bits(3 * n + 1, bh::to_int(second) as u32);
    let third = y * c(7) + (x * c(6) + z * c(8));
    to.set_bits(3 * n + 2, bh::to_int(third) as u32);
}

/// `scene::transform_int_points` (0x465829): `count` points of three whole numbers from
/// `from` turned by the matrix `m` into `to`. The registers it leaves (ecx -1, esi and edi
/// past the points) are read by no caller. Refuses a count of 0x80000000 (the count would
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
        m.set_bits(cell, st(value));
    }
}

/// A float made an 80-bit value (`fld dword; fstp tbyte`).
pub fn extended(f: u32) -> [u8; 10] {
    f80(ld(f))
}

/// An 80-bit value rounded to a float (`fld tbyte; fstp dword`).
pub fn rounded(v: [u8; 10]) -> u32 {
    st(from_f80(v))
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential, put_t80};

    /// The original's instructions for `turn_matrix3_x`.
    unsafe fn turn_x_ref(a: *mut u8, m: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m}]", "fldz",
                "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 8]",
                "fst dword ptr [{m} + 0xC]", "fstp dword ptr [{m} + 0x18]",
                "fst dword ptr [{m} + 0x10]", "fstp dword ptr [{m} + 0x20]",
                "fst dword ptr [{m} + 0x14]", "fchs", "fstp dword ptr [{m} + 0x1C]",
                a = in(reg) a,
                m = in(reg) m,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `turn_matrix3_y`.
    unsafe fn turn_y_ref(a: *mut u8, m: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x10]",
                "fldz", "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 0xC]",
                "fst dword ptr [{m} + 0x14]", "fstp dword ptr [{m} + 0x1C]",
                "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x20]",
                "fst dword ptr [{m} + 8]", "fchs", "fstp dword ptr [{m} + 0x18]",
                a = in(reg) a,
                m = in(reg) m,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `turn_matrix3_z`.
    unsafe fn turn_z_ref(a: *mut u8, m: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x20]",
                "fldz", "fst dword ptr [{m} + 8]", "fst dword ptr [{m} + 0x14]",
                "fst dword ptr [{m} + 0x18]", "fstp dword ptr [{m} + 0x1C]",
                "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x10]",
                "fst dword ptr [{m} + 4]", "fchs", "fstp dword ptr [{m} + 0xC]",
                a = in(reg) a,
                m = in(reg) m,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// `extended`'s instructions.
    unsafe fn extended_ref(f: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{f}]", "fstp tbyte ptr [{o}]",
                f = in(reg) f,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// `rounded`'s instructions.
    unsafe fn rounded_ref(v: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld tbyte ptr [{v}]", "fstp dword ptr [{o}]",
                v = in(reg) v,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The angle (an 80-bit value, dwords 0 .. 3), then the nine floats at dword 4. The sine
    /// and cosine are libm's: one float ulp apart is allowed.
    fn turn_test(name: &str, seed: u64, reference: unsafe fn(*mut u8, *mut u8), stores: Stores) {
        differential::<13>(
            name,
            6000,
            seed,
            [Kind::Ulp1; 13],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                put_t80(m, 0, rng.angle());
                m[2] = 0;
                m[3..].fill(0xAAAA_AAAA);
            },
            |base| unsafe { reference(base, base.add(12)) },
            |base| unsafe {
                let m: [crate::session::Dword; 9] = std::array::from_fn(|n| {
                    crate::session::Dword::new(base.add(12 + 4 * n).cast::<u32>().read())
                });
                turn(stores, base.cast::<[u8; 10]>().read_unaligned(), &m);
                for (n, c) in m.iter().enumerate() {
                    base.add(12 + 4 * n).cast::<u32>().write(c.get());
                }
            },
        );
    }

    #[test]
    fn turn_x_matches_the_assembly() {
        turn_test("turn_matrix3_x", 7, turn_x_ref, TURN_X);
    }

    #[test]
    fn turn_y_matches_the_assembly() {
        turn_test("turn_matrix3_y", 8, turn_y_ref, TURN_Y);
    }

    #[test]
    fn turn_z_matches_the_assembly() {
        turn_test("turn_matrix3_z", 9, turn_z_ref, TURN_Z);
    }

    #[test]
    fn extended_and_rounded_matches_the_assembly() {
        // a float, its 80-bit value, the 80-bit value rounded back
        differential::<8>(
            "extended and rounded",
            8000,
            10,
            [Kind::Exact; 8],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                m[0] = rng.fl();
                put_t80(m, 16, rng.fl());
            },
            |base| unsafe {
                extended_ref(base, base.add(4));
                rounded_ref(base.add(16), base.add(28));
            },
            |base| unsafe {
                base.add(4)
                    .cast::<[u8; 10]>()
                    .write_unaligned(extended(base.cast::<u32>().read()));
                base.add(28)
                    .cast::<u32>()
                    .write_unaligned(rounded(base.add(16).cast::<[u8; 10]>().read_unaligned()));
            },
        );
    }
}
