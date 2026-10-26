//! `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0 and the 5 x 5 cells around the view's grid place by `race::cell_sound_spots` (see `race/sound_spots.rs`).
//!
//! In:  nothing
//! Out: ecx the last cell, ebx / esi as the last cell leaves them
//!
//! Caller: `sub_4547AE` (0x45473D). Refuses (before anything) a cell past the grid, or no
//! records, faces or points when a cell has a record.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::all_regs;
use crate::race::sound_spots::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { find_sound_spots(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
