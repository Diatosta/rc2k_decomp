//! `race::prepare_course_view` (0x4548A8): `dword_63F17C` 0x10, `race::face_hide_mask`,
//! `race::course_grid`; with `dword_621480` set, `dword_63F17C` 0x10 again,
//! `race::checkpoint_colours` (with two checkpoints or more, `g_checkpointCount`) and
//! `race::mark_far_cells`.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455812), each view drawn. Refuses
//! (before anything) no course records, or what `probe_ground` refuses (it runs first in
//! the Rust version: nothing it reads is written before it). Shadow mode: by state (the
//! probe's regions, the mask, the grid and past it, the checkpoint colours, the far cells'
//! dwords). Self-test: on the game's state, `dword_621480` as it is and flipped; all put
//! back (skipped without a stage).

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_621480

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { prepare_course_view(r)? }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
