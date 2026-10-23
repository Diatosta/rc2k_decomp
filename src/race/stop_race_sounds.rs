//! `race::stop_race_sounds` (0x45D28E): `race::reset_race_sounds` (a `call` and a `retn`;
//! "stop sounds").
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `sub_444AC7`, `sub_444F7A` twice, `sub_45C53B`'s
//! neighbour (no flags read). It begins with its call: private copy, the call relocated,
//! manual patch. Shadow mode runs the original alone (it sends commands). Self-test: a
//! copy whose call goes to a stand-in that notes it, against the Rust version's call.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// `race::stop_race_sounds`: its one call.
pub unsafe fn stop_race_sounds(entry: &Regs, part: Part) -> Result<Regs, crate::errors::Refused> {
    let mut g = *entry;
    part(0x10, &mut g)?;
    Ok(g)
}

fn game(_: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { reset_race_sounds(r, &SoundVars::game(), &mut sound_part) }.map(|_| ())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { stop_race_sounds(r, &mut game) }?;
    Ok(())
}
