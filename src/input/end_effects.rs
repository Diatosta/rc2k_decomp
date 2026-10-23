//! `input::end_effects` (0x40F2B8): the race's force feedback ended: both records'
//! effects asked to stop (`input::stop_effects`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 2, the race's
//! end). Its call through `part`. A call in its first five bytes: private copy (8 bytes,
//! the call relocated), manual patch. Shadow mode runs the original alone. Self-test: a
//! copy whose call goes to a noting stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::input::effects::{STOP, effects_entry};

/// The Rust version.
pub fn end_effects(r: &mut Regs) -> Result<(), Refused> {
    let mut g = *r;
    crate::input::effects::game_part(STOP, &mut g)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, end_effects)
}
