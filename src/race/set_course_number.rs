//! `race::set_course_number` (0x43944F): the course's number put into the stage's paths.
//!
//! In:  al = the number
//! Out: every register preserved
//!
//! See `race/level_init.rs`: the tens and the units digit into the nine path strings that
//! hold it. Two callers (`sub_44426F` and one more). Self-test: the game's strings, numbers
//! 0 .. 255 by steps, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::level_init::set_course_number;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_course_number(r.eax) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
