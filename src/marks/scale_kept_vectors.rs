//! `marks::scale_kept_vectors` (0x4C15F4): three kept vectors scaled by their kept factors
//! (`marks::scale_vectors_by`' pattern): `flt_76927A` .. = `dword_76926E` .. by
//! `dword_76926A`, `flt_7692CE` .. = `dword_7692C2` .. by `dword_7692BE`, `flt_7692B2` ..
//! = `dword_7692A6` .. by `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the twelve inputs by lot, compared on the nine
//! results; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is
//! `scale_vectors_by::scale_triples_plain`, through `rc2k_core::marks::view::scaled` (plain
//! floats; the x87 version stays for the Rust callers in unconverted code). Shadow mode and
//! the self-test compare within `service::floats`' tolerances (the self-test at both test
//! precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::marks::scale_vectors_by::{results_of, scale_triples, scale_triples_plain};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::at;

/// (the factor, the vector, the result).
const PAIRS: [(usize, usize, usize); 3] = [
    (rva(0x76926A), rva(0x76926E), rva(0x76927A)),
    (rva(0x7692BE), rva(0x7692C2), rva(0x7692CE)),
    (rva(0x7692A2), rva(0x7692A6), rva(0x7692B2)),
];

/// The Rust version (see the header).
pub unsafe fn scale_kept_vectors() {
    unsafe { scale_triples(&PAIRS) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { scale_triples_plain(&PAIRS) };
    Ok(Left::nothing())
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    results_of(&PAIRS).into_iter().map(at).collect()
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
