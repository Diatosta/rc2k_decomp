//! `race::store_results_by_mode` (0x449957): the race's results kept by
//! `dword_62389C`: 2 the first block's +0x8C, +0x654 (also `dword_6051BC`) and +0x88;
//! 0 the two players' +0x88 and `g_stageCarTotal` x 100 less each (held to 0 ..
//! 300000); 1 and 3 the two +0x88s; others nothing.
//!
//! In:  nothing
//! Out: eax = the last value (mode 0: ebx the stage held to 0 .. 0x4C); the rest
//!      preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_4486FB` (a `mov` follows). Shadow mode:
//! the Rust version, the results put back, the original's, compared. Self-test: modes 0
//! .. 4, times round the bounds, the game's variables put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { store_results_by_mode(r) };
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
