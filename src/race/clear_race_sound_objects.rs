//! `race::clear_race_sound_objects` (0x415C6D): `race::silence_race_sounds`;
//! `dword_514F78`, `dword_514F7C`, `dword_514F8C` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. Four callers (`sub_4843EC` and menus; a call follows each).
//! Its call at +1: private copy, the call relocated, manual patch. Shadow mode runs the
//! original alone (it sends commands). Self-test: a copy whose call goes to a stand-in,
//! against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::sound_tables::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clear_race_sound_objects(r) }?;
    Ok(())
}
