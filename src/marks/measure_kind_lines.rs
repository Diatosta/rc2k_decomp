//! `marks::measure_kind_lines` (0x4D1225): faces esi and edi, block ebp: the lines of the
//! block's 0x11 kinds (+0x130, sums +0x1B8, counts +0x240).
//!
//! In:  esi, edi the two faces, ebp the block
//! Out: eax 0x11, ebx and ecx the last differences
//!
//! See `marks/lines.rs`. One caller, `marks::measure_lines`. Self-test: the original and
//! the Rust version on a scene of the test's (faces, points, blocks by lot), put back
//! afterwards; at the test precisions (the sums within one at extended).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { measure_kind_lines_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
