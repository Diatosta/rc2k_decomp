//! `race::count_checks` (0x445AAB): for each checkpoint, how many up to it are checks
//! (their face's byte +0x26 neither 0 nor 1).
//!
//! In:  nothing
//! Out: with two checkpoints or more, ebp the last count, ebx the checkpoint count, esi
//!      the last face; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_445772` ("checks"). Refuses more checkpoints
//! than the lists hold. Self-test: faces and checkpoints of the test's, the game's put
//! back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        count_checks(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
