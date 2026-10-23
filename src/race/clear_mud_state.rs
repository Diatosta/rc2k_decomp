//! `race::clear_mud_state` (0x449E01): the mud's 20 state dwords 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/mud.rs`. One caller, `race::init_mud`. Self-test: the game's variables
//! filled with 0xAA, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::mud::clear_mud_state;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_mud_state() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
