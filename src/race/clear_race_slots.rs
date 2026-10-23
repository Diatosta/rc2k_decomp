//! `race::clear_race_slots` (0x4621CD): the eight `g_raceSlots` (0x71BAC8, 0x78 each)
//! marked unused (first dword 0).
//!
//! In:  nothing
//! Out: edi = past the last record, ecx = 0; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init), twice. Shadow mode: the
//! Rust version, the slots put back, the original's, compared. Self-test: the game's slots
//! filled with values of the test's, both versions, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, EDI, Regs};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { clear_race_slots(r) };
    Ok(Left::nothing().with(EDI, g.edi).with(ECX, g.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
