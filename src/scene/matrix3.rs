//! The game's 3 x 3 matrix helpers at 0x46574E .. 0x46587C (IDA, 2026-10-16), used by the
//! marks: floats, rows of 12 bytes. The three that make a turn (0x465783, 0x4657A6, 0x4657C9)
//! take their angle in st(0) and pop it.
//!
//! Plain Rust since 2026-10-24 (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions are `asm_reference` in the tests.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float::{ONE, X, f80, from_f80, ld, st};
use crate::scene::vector3::write_floats;
use crate::scene::x87_mem::{fld80, fstp};

/// `scene::identity_matrix3` (0x46574E): the matrix at esi the identity (1.0 from `fld1`).
/// Every register kept.
pub fn identity_matrix3(at: u32) {
    write_floats(at, &rc2k_core::math::vector::IDENTITY);
}

/// `scene::multiply_matrices3` (0x4657EC): `out` = `a` x `b`, column by column, each written
/// as made (so an `out` that is one of the two behaves as there). The registers the
/// original leaves (ecx and edx 0, edi and ebp 12 on) are read by no caller (liveness at the
/// five call sites, all `marks`', which are Rust).
pub fn multiply_matrices3(a: u32, b: u32, out: u32) {
    for column in 0..3 {
        for row in 0..3 {
            x87::row_times_column_3(
                crate::pointers::host(a + 12 * row) as *const u32,
                crate::pointers::host(b + 4 * column) as *const u32,
                crate::pointers::host(out + 12 * row + 4 * column) as *mut u32,
            );
        }
    }
}

/// `scene::transform_int_points` (0x465829): `count` points of three whole numbers from
/// `from` turned by the matrix at `matrix` into `to` (`math::x87::turn_int_point`). The
/// registers it leaves (ecx -1, esi and edi past the points) are read by no caller. Refuses
/// a count of 0x80000000 (the count would run away).
pub fn transform_int_points(from: u32, to: u32, count: u32, matrix: u32) -> Result<(), Refused> {
    if count == 0x8000_0000 {
        return Err(BadInput("a count of 0x80000000: the original runs away").into());
    }
    for n in 0..(count as i32).max(0) as u32 {
        x87::turn_int_point(
            crate::pointers::host(from.wrapping_add(12 * n)) as *const i32,
            crate::pointers::host(matrix) as *const u32,
            crate::pointers::host(to.wrapping_add(12 * n)) as *mut i32,
        );
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

/// `scene::turn_matrix3_x` (0x465783), `_y` (0x4657A6), `_z` (0x4657C9): the matrix at esi
/// a turn by the angle about that axis (the original's `fsincos` and stores, in its order).
/// Every register kept. Refuses an angle `fsincos` cannot take.
pub fn turn_matrix3(axis: Axis, at: u32, angle: [u8; 10]) -> Result<(), Refused> {
    turnable(&angle)?;
    let a = angle.as_ptr();
    let m = crate::pointers::host(at);
    match axis {
        Axis::X => turn_x(a, m),
        Axis::Y => turn_y(a, m),
        Axis::Z => turn_z(a, m),
    }
    Ok(())
}

/// `fld tbyte; fsincos; fld1; fstp ..; fldz; fst ..` for the turn about x: the identity's
/// row and column of x, the cosine on the diagonal, the sine and minus the sine across.
fn turn_x(a: *const u8, m: *mut u8) {
    let t0 = fld80(a, 0x0);
    let t1 = t0.sin();
    let t2 = t0.cos();
    let t3 = ONE;
    fstp(m, 0x0, t3);
    let t4 = X(0.0);
    fstp(m, 0x4, t4);
    fstp(m, 0x8, t4);
    fstp(m, 0xC, t4);
    fstp(m, 0x18, t4);
    fstp(m, 0x10, t2);
    fstp(m, 0x20, t2);
    fstp(m, 0x14, t1);
    let t5 = -t1;
    fstp(m, 0x1C, t5);
}

/// The turn about y (see `turn_x`).
fn turn_y(a: *const u8, m: *mut u8) {
    let t0 = fld80(a, 0x0);
    let t1 = t0.sin();
    let t2 = t0.cos();
    let t3 = ONE;
    fstp(m, 0x10, t3);
    let t4 = X(0.0);
    fstp(m, 0x4, t4);
    fstp(m, 0xC, t4);
    fstp(m, 0x14, t4);
    fstp(m, 0x1C, t4);
    fstp(m, 0x0, t2);
    fstp(m, 0x20, t2);
    fstp(m, 0x8, t1);
    let t5 = -t1;
    fstp(m, 0x18, t5);
}

/// The turn about z (see `turn_x`).
fn turn_z(a: *const u8, m: *mut u8) {
    let t0 = fld80(a, 0x0);
    let t1 = t0.sin();
    let t2 = t0.cos();
    let t3 = ONE;
    fstp(m, 0x20, t3);
    let t4 = X(0.0);
    fstp(m, 0x8, t4);
    fstp(m, 0x14, t4);
    fstp(m, 0x18, t4);
    fstp(m, 0x1C, t4);
    fstp(m, 0x0, t2);
    fstp(m, 0x10, t2);
    fstp(m, 0x4, t1);
    let t5 = -t1;
    fstp(m, 0xC, t5);
}

/// A float made an 80-bit value (`fld dword; fstp tbyte`).
pub fn extended(f: u32) -> [u8; 10] {
    f80(ld(f))
}

/// An 80-bit value rounded to a float (`fld tbyte; fstp dword`).
pub fn rounded(v: [u8; 10]) -> u32 {
    st(from_f80(v))
}

#[cfg(test)]
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
    fn turn_test(
        name: &str,
        seed: u64,
        reference: unsafe fn(*mut u8, *mut u8),
        plain: unsafe fn(*const u8, *mut u8),
    ) {
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
            |base| unsafe { plain(base, base.add(12)) },
        );
    }

    #[test]
    fn turn_x_matches_the_assembly() {
        turn_test("turn_matrix3_x", 7, turn_x_ref, turn_x);
    }

    #[test]
    fn turn_y_matches_the_assembly() {
        turn_test("turn_matrix3_y", 8, turn_y_ref, turn_y);
    }

    #[test]
    fn turn_z_matches_the_assembly() {
        turn_test("turn_matrix3_z", 9, turn_z_ref, turn_z);
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
