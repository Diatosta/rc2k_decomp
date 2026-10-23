//! `menu::load_welcome_picture` (0x4684BD): the welcome screen's background (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::welcome_loop` (its background). Shadow mode runs the original alone (it
//! loads a file). Self-test: `stage_view_steps::test_picture`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { crate::menu::stage_view_steps::load_welcome_picture(r, part) },
        crate::menu::stage_view_steps::picture_part,
    )
}
