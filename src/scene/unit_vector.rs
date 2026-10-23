//! `scene::unit_vector` (0x420EF8): three floats made one long.
//!
//! In:  ecx = the three floats, edx = where the result goes (may be the same place)
//! Out: every register preserved (the flags are the last compare's; no caller reads them)
//!
//! The length (squares added, the last two first, `fsqrt`) is left in `dword_62400C`.
//! A length that compares equal to `flt_5C5D90` (0; also a length that is no number)
//! gives three zeros; else each float over the length, a minus zero made zero.
//! Four callers.
//! Refuses a null pointer.
//! Self-test: vectors by lot, apart and in place.
//!
//! Converted 2026-10-21: the hook's Rust version is `unit_vector_plain`, through
//! `rc2k_core::scene::matrices::unit_vector` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::scene::view_matrix;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ecx == 0 || r.edx == 0 {
        return Err(BadInput("a null vector").into());
    }
    unsafe { view_matrix::unit_vector_plain(r.ecx as usize, r.edx as usize) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
