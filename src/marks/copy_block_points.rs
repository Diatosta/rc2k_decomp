//! `marks::copy_block_points` (0x4D0A82): block ebp: its object's points copied to its +0x8C.
//!
//! In:  ebp the block
//! Out: eax the last dword copied (the count with none), esi and edi past the points
//!
//! See `marks/lines.rs`. One caller, `marks::set_up_blocks`. Refuses an object past 0xFF.
//! Self-test: the original and the Rust version on a scene of the test's (0, 1, 3 and 0x20
//! points of either object), the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { copy_block_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
