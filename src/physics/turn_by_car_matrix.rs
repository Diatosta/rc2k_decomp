//! `physics::turn_by_car_matrix` (0x43F14E): the three floats at esi turned and placed by
//! `g_crashMatrix` (0x71A08C, the work copy's +0xB4; rows 0x10 apart, the place at +0x30)
//! with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! In:  esi the point
//! Out: the point turned; every register kept
//!
//! `physics::turn_by_inverse` (0x43F118) is the same with the matrix at 0x71A9A4
//! (`g_carInverseMatrix`, made by `sub_43F16C` with `scene::invert_turn`).
//! Callers (6): the cars' physics under `sub_4454D2`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::cells::Cells;
use crate::physics::car_store::CAR;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

// g_crashMatrix

/// The point `v` turned by the matrix `m`, as the two functions do it: `fld z; fld y; fld
/// x`, `rotate_point`, `fstp x; fstp y; fstp z`.
pub fn turn_cells(m: &(impl Cells + ?Sized), v: &(impl Cells + ?Sized)) {
    let point = std::array::from_fn(|k| extended(v.bits(k)));
    let out = rotate_point(m, point);
    for (k, e) in out.iter().enumerate() {
        v.set_bits(k, rounded(*e));
    }
}

/// The Rust version.
pub fn turn_by_car_matrix(v: &(impl Cells + ?Sized)) {
    turn_cells(&CAR.crash_matrix, v);
}
