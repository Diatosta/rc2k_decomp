//! `marks::set_line_turn_back` (0x4BF466): the 2 x 2 turn of the line at esi (dx, dy) into
//! `flt_82E396` .. `flt_82E3A6`.
//!
//! In:  esi the line (+0, +4 to +0xC, +0x10)
//! Out: every register kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BEFD2` (twice). Self-test: the original and the
//! Rust version on lines by lot, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::set_line_turn_plain`, through
//! `rc2k_core::marks::turns::line_turn` (plain floats; the x87 version stays for the Rust
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
    unsafe { set_line_turn_plain(r.esi, true) };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
