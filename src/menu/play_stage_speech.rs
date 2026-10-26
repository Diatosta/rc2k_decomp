//! `menu::play_stage_speech` (0x47F437): the stage's speech in the championship (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop` and `next_stage_loop` (after the first frame) and the stage pages' steps. Shadow mode runs the original alone (it starts a stream). Self-test: `stage_view_steps::test_speech`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { crate::menu::stage_view_steps::play_stage_speech(r, part) },
        crate::menu::stage_view_steps::speech_part,
    )
}
