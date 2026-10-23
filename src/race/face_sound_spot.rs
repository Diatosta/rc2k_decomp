//! `race::face_sound_spot` (0x45C6ED): face ebx as a sound spot (kind 0xFF, object 0x3F, sound 1 .. 0x15, room left) (see `race/sound_spots.rs`).
//!
//! In:  ebx = the face
//! Out: esi, edi, eax .. ecx as it leaves them (inside its caller's `pusha`)
//!
//! Caller: `race::cell_sound_spots`. Shadow mode: by state. Self-test: a test course
//! (`SpotStage`: faces, records, points, the grid's cells), every test face, the spots full and not; all put back.
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
    Ok(all_regs(&unsafe { face_sound_spot(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
