//! `race::start_race_sounds` (0x40D592): the race's sounds started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sounds.rs`. Callers: `race::wake_car_sounds` and 0x41D855. Refuses a sound
//! object missing. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! two calls note themselves, the game's objects' flags, `dword_514F7C` over 0 .. 9 and -1,
//! put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::sounds::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { start_race_sounds(r, &mut game_part) }?;
    Ok(())
}
