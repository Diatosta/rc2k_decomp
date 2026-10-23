//! `race::sort_checkpoints` (0x4458A7): the best line's checkpoint faces found and
//! chained in order.
//!
//! In:  nothing
//! Out: eax the last checkpoint (count - 1) with a chain; ebx, ecx, edx, esi, ebp as the
//!      searches leave them (see `race/best_line.rs`)
//!
//! Callers: `sub_445772` (the best line) and `sub_459BCA`. Refuses no faces or 0x100000 or
//! more, and more line faces than the list holds. Manual patch, private copy (a call at
//! the start). Self-test: faces of the test's (chains whole, broken, without a start, too
//! few), the game's put back.
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
        sort_checkpoints(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
