//! `scene::invert_turn` (0x420CC2): a turning matrix undone.
//!
//! In:  esi = the matrix, edi = where the result goes
//! Out: every register preserved
//!
//! The matrix's nine floats (three rows of three) are copied to `flt_5C7050` ..; their
//! nine cofactors (each a product less a product, every other one negated) are made at
//! `flt_5C7074` .. and stored the other way round: the result's rows are the cofactors
//! of the matrix's columns. No division by the determinant: `scene::square_matrix` on
//! the result makes its rows one long instead. The result's fourth row and column are
//! what that function copies.
//! Six callers.
//! Refuses a null matrix.
//! Self-test: matrices by lot, the result apart and in the matrix's own place.
//!
//! Converted 2026-10-21: the hook's Rust version is `invert_turn_plain`, through
//! `rc2k_core::scene::matrices::invert_turn` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::scene::view_matrix::{self};
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 {
        return Err(BadInput("a null matrix").into());
    }
    unsafe { view_matrix::invert_turn_plain(r.esi as usize, r.edi as usize) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
