//! `marks::reset_mask_cursor` (0x4BE0A3): the masks' cursor at the work block, the runs at
//! their start, their count 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! See `marks/masks.rs`. One caller, `sub_4BDE9A`. Self-test: the original and the Rust
//! version, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_mask_cursor() };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
