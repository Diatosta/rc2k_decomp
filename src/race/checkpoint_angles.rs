//! `race::checkpoint_angles` (0x4459DF): each link of the best line's angle.
//!
//! In:  nothing
//! Out: eax the angle at 2; with two checkpoints or more, ebx the count; the rest
//!      preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("angles") and `sub_459BCA`. Refuses more
//! checkpoints than the lists hold. Self-test: checkpoints of the test's, the game's put
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
        checkpoint_angles(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
