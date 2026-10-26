//! `race::eax_environment` (0x45C7EE): the EAX sound environment's sum: `g_eaxEnvironment`
//! 0.0 and `dword_6D1AF0` 0, then every face of the course records in the 3 x 3 grid cells
//! around the middle (from `race::hold_to_limit(dword_60A87C)` - 1 on each axis; numbers
//! below 0x0FFFFFFF) weighed by `race::eax_face_weight` (`sub_45C84C` per cell, covered
//! here: its only caller is this function).
//!
//! In:  nothing
//! Out: eax the half, ecx the last cell, ebx and esi as the last cell left them (a `call`
//!      follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x45581C), each view drawn.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { eax_environment(r) }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
