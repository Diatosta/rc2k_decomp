//! `scene::set_turn_x` (0x426FE1): the turning matrix about the x axis for an angle.
//!
//! In:  edx = the angle, sixteen bits a turn
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The sine and cosine come from the game's two tables of 0x1000 floats (the angle's
//! low four bits dropped), are kept in `dword_624148` and `dword_62414C`, and go to the matrix
//! `TURN_X_RVA`: +0x14 the cosine, +0x18 the sine, +0x24 the sine negated, +0x28 the cosine.
//! Its other floats stay as they are.
//! 12 callers.
//! Self-test: 41 angles on the game's matrix, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_turn_x_plain`, through
//! `rc2k_core::scene::matrices::set_table_turn`; compared exactly as before (the earlier
//! version stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::matrix::{self};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::set_turn_x_plain(r.edx) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
