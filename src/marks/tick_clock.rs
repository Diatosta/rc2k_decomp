//! `marks::tick_clock` (0x4D2BDC): the marks' clock, started or moved on by
//! `g_sceneClock`.
//!
//! In:  nothing
//! Out: the first time every register preserved; then eax a sixteenth of the ticks, ebx
//!      the clock before, esi `g_marksSettings`
//!
//! See `marks/setup.rs`. Callers: `marks::init_marks` (step 15) and `marks::run_frame`
//! (each frame). Self-test: the original and the Rust version on the game's variables with
//! clocks, settings and a start of the test's, at the test precisions, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { tick_clock_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
