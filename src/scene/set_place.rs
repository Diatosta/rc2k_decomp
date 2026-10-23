//! `scene::set_place` (0x4271AE): three whole numbers as the place of the matrix
//! `flt_5EA04C`.
//!
//! In:  eax, ebx, ecx = x, y, z
//! Out: every register preserved
//!
//! Each as a float (`fild`, `fstp`) to the matrix's fourth row (`flt_5EA07C` ..); the
//! work variable `dword_62400C` is left holding z.
//! 20 callers.
//! Self-test: 24 places on the game's matrix, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_place_plain`, through
//! `rc2k_core::scene::matrices::set_place`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
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
    unsafe { matrix::set_place_plain([r.eax, r.ebx, r.ecx]) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
