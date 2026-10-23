//! `marks::scale_rows3` (0x4BF498): the three rows of three floats at esi each made a set
//! length (`flt_89CA24` .. `flt_89CA2C`).
//!
//! In:  esi the rows
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (four). A zero row gives NaNs, as there.
//! Self-test: the original and the Rust version on rows by lot (a zero one among them).
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::scale_rows3_plain`, through
//! `rc2k_core::marks::turns::scale_rows` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { scale_rows3_plain(r.esi) };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
