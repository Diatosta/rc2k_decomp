//! `race::set_both_counters` (0x43A987): the first two car blocks' +0x71C eax, +0x730 0,
//! +0x72C 0x12C (by their fixed addresses).
//!
//! In:  eax = the value
//! Out: every register preserved
//!
//! See `race/frame.rs`. Six callers in `sub_43A649`. Shadow mode: the Rust version, the
//! dwords put back, the original's, compared. Self-test: the game's two blocks, values,
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_both_counters(r.eax) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
