//! `scene::set_fine_turn_y` (0x42710C): the turning matrix about the y axis for an angle of
//! 32 bits a turn.
//!
//! In:  edx = the angle, a whole turn being 2^32
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The angle as a whole number (signed) over `flt_5C5D98` (65536) twice and over
//! `flt_5C5DD4` (1 / 2 pi), then `fsincos`: the instruction itself, as its result is
//! the processor's. The pair is kept in `dword_624150 and dword_624154`, the angle in
//! `dword_62400C`, and goes to `g_turnYMatrix`: +0 the cosine, +8 the sine negated, +0x20 the sine, +0x28 the cosine.
//! The matrix's other floats stay as they are.
//! Self-test: 48 angles on the game's matrix, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `view_matrix::set_fine_turn_plain`,
//! through `rc2k_core::scene::matrices::set_fine_turn` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::view_matrix::{self, Axis};
use crate::screens::harness::Left;
use crate::service::floats::*;

const AXIS: Axis = Axis::Y;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { view_matrix::set_fine_turn_plain(AXIS, r.edx) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
