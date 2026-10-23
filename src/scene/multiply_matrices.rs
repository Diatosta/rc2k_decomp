//! `scene::multiply_matrices` (0x42720D): one matrix times another, as far as a point needs it.
//!
//! In:  esi, edi = two matrices, ebp = where the result goes
//! Out: ecx = 0, edx = 0; everything else preserved
//!
//! Column by column for the first three columns, and in each row by row: the row's
//! first three floats of the first matrix times that column's first three of the
//! second, the last two products added first; then the second's fourth-row float of
//! that column added to the result's. The result's fourth column is left as it is.
//! (The x87 version: the original's instructions.)
//! 75 callers.
//! Refuses a null matrix.
//! Self-test: matrices of the test's by lot, also with the result in the place of
//! each of the two.
//!
//! Converted 2026-10-21: the hook's Rust version is `multiply_matrices_plain`, through
//! `rc2k_core::scene::matrices::multiply` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::scene::matrix::{self};
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 || r.ebp == 0 {
        return Err(BadInput("a null matrix").into());
    }
    Ok(unsafe { matrix::multiply_matrices_plain(r.esi as usize, r.edi as usize, r.ebp as usize) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
