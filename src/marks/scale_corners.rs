//! `marks::scale_corners` (0x4C6420): the 0x100 corner records' word pairs scaled into
//! `g_marksCornerSlopes`, with each record's slope.
//!
//! In:  nothing
//! Out: eax, ebx the last pair, esi 0, ebp and edi the two sides
//!
//! See `marks/blocks.rs`. Callers: `marks::allocate_links` and `sub_4C5409`. Self-test:
//! the original and the Rust version on the game's tables with corners of the test's (no
//! side 0), put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_corners_plain`, through
//! `rc2k_core::marks::scales::scale_corner` (plain floats; the x87 version stays for
//! `marks::allocate_links`' part). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::blocks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_corners_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
