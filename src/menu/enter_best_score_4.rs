//! `menu::enter_best_score_4` (0x4A871C): as `menu::enter_best_score` with the score +0xCC into group 4.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A91BF`. Refuses as `enter_best_score`. Self-test: the original and the Rust version on the game's
//! tables, scores by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn enter_best_score_4(entry: &Regs) -> Result<Regs, Refused> {
    enter_best(Some(4), 0xCC)?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { enter_best_score_4(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
