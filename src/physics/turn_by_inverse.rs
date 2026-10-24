//! `physics::turn_by_inverse` (0x43F118): the three floats at esi turned and placed by
//! `g_carInverseMatrix` (0x71A9A4, the work copy's +0x9CC, made from `g_crashMatrix` by
//! `sub_43F16C`) with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! In:  esi the point
//! Out: the point turned; every register kept
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers (13): the cars'
//! physics under `sub_4454D2`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::{game_addr, rva};
use crate::physics::turn_by_car_matrix::turn_point_by;

pub const INVERSE_RVA: usize = rva(0x71A9A4); // g_carInverseMatrix

/// The Rust version.
pub unsafe fn turn_by_inverse(v: u32) {
    unsafe { turn_point_by(game_addr(INVERSE_RVA) as u32, v) }
}
