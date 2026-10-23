//! `physics::store_motion_length` (0x43BD2D): `dword_71A19C` = the length of the three
//! floats at `dword_71A170` (`physics::vector_length`, then `fstp`: rounded to a float).
//! Both are the work copy's (+0x198, +0x1C4); that the vector is the car's motion is a
//! reading (it sits next to `g_carSpeed`), not seen.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_43B635` (0x43B644, 0x43B94D), the car's physics step under
//! `sub_4454D2`.
//! Self-test: a copy of the original on test variables (its call to the
//! `vector_length` hook kept) against the converted version, vectors by lot, exact at
//! double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::read_floats;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::motion::motion_length;

const MOTION_RVA: usize = rva(0x71A170); // dword_71A170: three floats
const LENGTH_RVA: usize = rva(0x71A19C); // dword_71A19C

/// The Rust core on the vector at `v` and the length at `out`.
pub unsafe fn store_length(v: u32, out: u32) {
    unsafe { (out as *mut u32).write_unaligned(rounded(vector_length(v))) }
}

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the vector at
/// `v` and the length at `out`.
// Converted: `rc2k_core::physics::motion::motion_length`.
pub unsafe fn store_length_plain(v: u32, out: u32) {
    unsafe { (out as *mut u32).write_unaligned(motion_length(read_floats(v)).to_bits()) }
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::keep_motion_vectors`).
pub unsafe fn store_motion_length() {
    unsafe { store_length(game_addr(MOTION_RVA) as u32, game_addr(LENGTH_RVA) as u32) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_length_plain(game_addr(MOTION_RVA) as u32, game_addr(LENGTH_RVA) as u32) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
