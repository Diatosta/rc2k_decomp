//! The game's 3 x 3 matrix helpers at 0x46574E .. 0x46587C (IDA, 2026-10-16), used by the
//! marks: floats, rows of 12 bytes. The three that make a turn (0x465783, 0x4657A6,
//! 0x4657C9) take their angle in st(0) and pop it: their hooks use `angle_hook!`, whose
//! thunk keeps the angle (80 bits) for the Rust version and loads it again for the original.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87::x87;
use crate::scene::vector3::write_floats;

/// `scene::identity_matrix3` (0x46574E): the matrix at esi the identity (1.0 from `fld1`).
/// Every register kept.
pub unsafe fn identity_matrix3(at: u32) {
    unsafe { write_floats(at, &rc2k_core::math::vector::IDENTITY) };
}

/// `scene::multiply_matrices3` (0x4657EC): `out` = `a` x `b`, column by column, each written
/// as made (so an `out` that is one of the two behaves as there). The registers the
/// original leaves (ecx and edx 0, edi and ebp 12 on) are read by no caller (liveness at the
/// five call sites, all `marks`', which are Rust).
pub unsafe fn multiply_matrices3(a: u32, b: u32, out: u32) {
    unsafe {
        for column in 0..3 {
            for row in 0..3 {
                x87::row_times_column_3(
                    (a + 12 * row) as *const u32,
                    (b + 4 * column) as *const u32,
                    (out + 12 * row + 4 * column) as *mut u32,
                );
            }
        }
    }
}

/// `scene::transform_int_points` (0x465829): `count` points of three whole numbers from
/// `from` turned by the matrix at `matrix` into `to` (`math::x87::turn_int_point`). The
/// registers it leaves (ecx -1, esi and edi past the points) are read by no caller. Refuses
/// a count of 0x80000000 (the count would run away).
pub unsafe fn transform_int_points(
    from: u32,
    to: u32,
    count: u32,
    matrix: u32,
) -> Result<(), Refused> {
    if count == 0x8000_0000 {
        return Err(BadInput("a count of 0x80000000: the original runs away").into());
    }
    unsafe {
        for n in 0..(count as i32).max(0) as u32 {
            x87::turn_int_point(
                from.wrapping_add(12 * n) as *const i32,
                matrix as *const u32,
                to.wrapping_add(12 * n) as *mut i32,
            );
        }
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
pub unsafe fn turn_matrix3(axis: Axis, at: u32, angle: [u8; 10]) -> Result<(), Refused> {
    turnable(&angle)?;
    let a = angle.as_ptr();
    match axis {
        Axis::X => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m}]", "fldz",
            "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 8]",
            "fst dword ptr [{m} + 0xC]", "fstp dword ptr [{m} + 0x18]",
            "fst dword ptr [{m} + 0x10]", "fstp dword ptr [{m} + 0x20]",
            "fst dword ptr [{m} + 0x14]", "fchs", "fstp dword ptr [{m} + 0x1C]";
            a = a, m = at
        ),
        Axis::Y => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x10]",
            "fldz", "fst dword ptr [{m} + 4]", "fst dword ptr [{m} + 0xC]",
            "fst dword ptr [{m} + 0x14]", "fstp dword ptr [{m} + 0x1C]",
            "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x20]",
            "fst dword ptr [{m} + 8]", "fchs", "fstp dword ptr [{m} + 0x18]";
            a = a, m = at
        ),
        Axis::Z => x87!(
            "fld tbyte ptr [{a}]", "fsincos", "fld1", "fstp dword ptr [{m} + 0x20]",
            "fldz", "fst dword ptr [{m} + 8]", "fst dword ptr [{m} + 0x14]",
            "fst dword ptr [{m} + 0x18]", "fstp dword ptr [{m} + 0x1C]",
            "fst dword ptr [{m}]", "fstp dword ptr [{m} + 0x10]",
            "fst dword ptr [{m} + 4]", "fchs", "fstp dword ptr [{m} + 0xC]";
            a = a, m = at
        ),
    }
    Ok(())
}

/// A float made an 80-bit value (`fld dword; fstp tbyte`).
pub fn extended(f: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let o = out.as_mut_ptr();
    x87!("fld dword ptr [{f}]", "fstp tbyte ptr [{o}]"; f = &f as *const u32, o = o);
    out
}

/// An 80-bit value rounded to a float (`fld tbyte; fstp dword`).
pub fn rounded(v: [u8; 10]) -> u32 {
    let mut out = 0u32;
    x87!("fld tbyte ptr [{v}]", "fstp dword ptr [{o}]"; v = v.as_ptr(), o = &mut out as *mut u32);
    out
}
