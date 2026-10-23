//! `physics::turn_by_inverse` (0x43F118): the three floats at esi turned and placed by
//! `g_carInverseMatrix` (0x71A9A4, the work copy's +0x9CC, made from `g_crashMatrix` by
//! `sub_43F16C`) with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! In:  esi the point
//! Out: the point turned; every register kept
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers (13): the cars'
//! physics under `sub_4454D2`.
//! Self-test: as `physics::turn_by_car_matrix`'s, on this function's copy.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::turn_by_car_matrix::{turn_point_by, turn_point_by_plain};
use crate::screens::harness::Left;

pub const INVERSE_RVA: usize = rva(0x71A9A4); // g_carInverseMatrix

/// The Rust version.
pub unsafe fn turn_by_inverse(v: u32) {
    unsafe { turn_point_by(game_addr(INVERSE_RVA) as u32, v) }
}

/// The converted version (`turn_point_by_plain`): the hook's.
pub unsafe fn turn_by_inverse_plain(v: u32) {
    unsafe { turn_point_by_plain(game_addr(INVERSE_RVA) as u32, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_inverse_plain(r.esi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}
