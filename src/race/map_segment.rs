//! `race::map_segment` (0x446036): a stretch of the best line, from one map anchor to the
//! next, placed on the map.
//!
//! In:  nothing (the anchors in `dword_6236C4`, `dword_6236C0`)
//! Out: see `race::best_line::map_segment`
//!
//! See `race/best_line.rs`. Callers: `race::map_checkpoints` (inside `pusha`) and the code
//! at 0x44601B (the whole line as one stretch). Refuses anchors past the lists. Self-test:
//! faces and checkpoints of the test's, the game's put back.
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
        map_segment(r)
    }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
