//! `race::set_up_race_sound_objects` (0x415C93): the race's sound objects set up: the
//! sound tables made ready (`race::prepare_sound_tables`), the players' and objects'
//! counts, the players' objects, each object's first dword -1 .. -8 and its state reset;
//! then `race::restart_race_sounds`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. Seven callers (`sub_4843EC` and menus; a call follows each).
//! It begins with its call at +1: private copy, both calls relocated, manual patch.
//! Refuses, before anything, a chosen car past 7 or a sound object missing. Shadow mode
//! runs the original alone (the audio thread reads the objects). Self-test: a copy whose
//! two calls go to stand-ins and whose eleven references to the objects and their table
//! are the test's, against the Rust version on the same: one and two players, mode 2, a
//! chosen car.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::sound_tables::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_up_race_sound_objects(r, &ObjectVars::game()) }?;
    Ok(())
}
