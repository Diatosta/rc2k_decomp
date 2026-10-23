//! `race::wake_car_sounds` (0x45D255): the cars' sound objects woken at the stage's loading.
//!
//! In:  nothing
//! Out: ebx = the cars' count and edi = the last object woken, when any is; every other
//!      register preserved
//!
//! See `race/sounds.rs`. Callers: `race::load_stage` and two more. Refuses 0x10000 cars or
//! more, a sound object missing. Shadow mode runs the original alone. Self-test: a
//! redirected copy whose call of `race::start_race_sounds` notes itself, the game's
//! objects' flags, `dword_71C3E0` 0 and 1, the cars 0, 1, 5 and 9, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::sounds::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { wake_car_sounds(r, &mut game_wake_part) }?;
        r.ebx = left.ebx;
        r.edi = left.edi;
    }
    Ok(())
}
