//! `scene::square_matrix` (0x420E94): a turning matrix's three rows made one long and at
//! right angles to each other again.
//!
//! In:  esi = the matrix (changed in place)
//! Out: every register preserved
//!
//! In the work rows `unk_5C6FD0`, `unk_5C6FE0`, `unk_5C6FF0`: the third = the matrix's
//! third row as a unit vector; the first = the matrix's second row crossed with it, as
//! a unit vector; the second = the third crossed with the first. Then sixteen dwords
//! from `unk_5C6FD0` on go to the matrix: the three rows with whatever their fourth
//! dwords hold, and the four dwords after them as the fourth row.
//! Four callers.
//! Refuses a null matrix.
//! Self-test: matrices by lot.
//!
//! Converted 2026-10-21: the hook's Rust version is `square_matrix_plain`, through
//! `rc2k_core::scene::matrices::square` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
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
    if r.esi == 0 {
        return Err(BadInput("a null matrix").into());
    }
    unsafe { view_matrix::square_matrix_plain(r.esi as usize) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
