//! `race::load_prefs` (0x445698): the stage's `localdat` read and its swaps (`load prefs`).
//!
//! In:  nothing
//! Out: as `swap_stage_texture` leaves them
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe {
        load_prefs(r, &mut crate::race::level_init::game_part)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
