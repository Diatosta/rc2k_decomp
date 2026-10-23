//! `marks::clear_slots` (0x4C967B): `init_marks`' step 11, the first dword of the 0x20
//! slots of `g_markSlots` 0.
//!
//! In:  nothing
//! Out: esi 0
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Manual patch with a private copy
//! (its loop jumps back into the first five bytes). Self-test: the slots filled with 0xAA,
//! put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_slots(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
