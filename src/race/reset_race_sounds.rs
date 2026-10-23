//! `race::reset_race_sounds` (0x40D378): the race's sounds reset: commands waited for,
//! command 7 (and its end of turn), waited again, sounds held back, the race clock
//! started; the eight race sound objects' 32 fields, eleven variables, the 0x48 voices'
//! first and sixth dwords, the EAX environment 1.0; command 2 (eax 3); `dword_5124E4` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `race::stop_race_sounds`, `sub_40D34A`, `sub_40D366`,
//! `sub_41CDC2`, `sub_41D85F`. Refuses, before anything, a sound object missing (the
//! original writes through 0). Shadow mode runs the original alone (it sends commands).
//! Self-test: a copy whose five sound calls note themselves and whose object table and
//! voices are the test's (the audio thread reads the game's), against the Rust version on
//! the same; the plain variables put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_r: &mut Regs) -> Result<(), crate::errors::Refused> {
    reset_race_sounds(&SoundVars::game())?;
    Ok(())
}
