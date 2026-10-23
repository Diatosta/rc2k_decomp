//! `race::cell_sound_spots` (0x45C6B8): each face of grid cell ecx's record by `race::face_sound_spot` (see `race/sound_spots.rs`).
//!
//! In:  ecx = the cell
//! Out: ebx past the last face, esi the record (or ebx the grid's entry); ecx kept
//!
//! Caller: `race::find_sound_spots` (25 places). Shadow mode: by state. Self-test: a test course
//! (`SpotStage`: faces, records, points, the grid's cells), each cell of the test grid; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::all_regs;
use crate::race::sound_spots::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { cell_sound_spots(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
