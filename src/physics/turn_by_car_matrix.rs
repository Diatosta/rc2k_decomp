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
//! Self-test: a copy of the original whose matrix is the test's (the car's is used all
//! through a frame; its call to `scene::rotate_point` goes to that hook), against the
//! Rust core on points and matrices by lot, under three precisions.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::game_addr;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

pub use crate::race::stage_prefs::CRASH_MATRIX_RVA; // g_crashMatrix

/// The point at `v` turned by the matrix at `m`, as the two functions do it: `fld z; fld
/// y; fld x`, `rotate_point`, `fstp x; fstp y; fstp z`.
pub unsafe fn turn_point_by(m: u32, v: u32) {
    unsafe {
        let p = v as *mut u32;
        let point = [
            extended(p.read_unaligned()),
            extended(p.add(1).read_unaligned()),
            extended(p.add(2).read_unaligned()),
        ];
        let out = rotate_point(m, point);
        p.write_unaligned(rounded(out[0]));
        p.add(1).write_unaligned(rounded(out[1]));
        p.add(2).write_unaligned(rounded(out[2]));
    }
}

/// The Rust version.
pub unsafe fn turn_by_car_matrix(v: u32) {
    unsafe { turn_point_by(game_addr(CRASH_MATRIX_RVA) as u32, v) }
}
