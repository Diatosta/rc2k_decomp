//! `race::share_sound_spots` (0x45C7B9): `dword_5165EC` = `dword_6D1DC4`, and that many
//! 0x14-byte spots copied from 0x6D1AF4 to 0x5165F0, where the sound side
//! (`sub_40E642`) reads them.
//!
//! In:  nothing
//! Out: ecx 0, esi past the spots (0x6D1AF4 with none), edi past the copy (a `call`
//!      follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45429B` (0x454742), each frame drawn. What a spot
//! is (made by `sub_45C6ED`) is a reading. Refuses (before anything) more than 36 (both
//! tables hold 36: 0x6D1AF4 runs to the count, 0x5165F0 to `unk_5168C0`).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { share_sound_spots() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
