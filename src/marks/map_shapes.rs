//! `marks::map_shapes` (0x4BE752): the pixel table at esi cleared and every shape at ebp
//! mapped into it from the corner table edi, rounding toward zero.
//!
//! In:  ebp the shapes, esi the table (0x8000 dwords), edi the corner table
//! Out: eax the count, ecx 0, esi and edi as the last shape leaves them
//!
//! See `marks/pixels.rs`. Callers: `marks::make_masks` (the two shapes). Refuses a count of
//! 0 or less or past 0x80, and rows or spans outside the table, before writing.
//! Self-test: the original and the Rust version on shapes and corner tables by lot with a
//! table of the test's, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::pixels::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { map_shapes(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
