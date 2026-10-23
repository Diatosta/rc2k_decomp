//! `race::surface_shares` (0x445E30): how much of the best line runs on each surface.
//!
//! In:  nothing
//! Out: with two checkpoints or more, ecx 16, edi past the shares, edx the count, eax, ebx,
//!      esi as the last face leaves them; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_445772` (after "replaycam2"). Refuses more
//! checkpoints than the lists hold. Self-test: faces and checkpoints of the test's (the
//! game's texture infos read as they are), the game's put back.
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
        surface_shares(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
