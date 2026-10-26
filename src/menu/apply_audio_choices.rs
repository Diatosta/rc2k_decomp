//! `menu::apply_audio_choices` (0x470766): the advanced audio page's work values through
//! `audio::choose_modes` and back (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! An item action of the advanced audio page. Shadow mode runs the original alone (the
//! modes' choice writes the log). Self-test: `option_steps::test_audio`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::option_steps::{apply_audio_choices, audio_part};
use crate::menu::parts::parted_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(
        r,
        &|r, part| unsafe { apply_audio_choices(r, part) },
        audio_part,
    )
}
