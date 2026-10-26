//! `sound::race_turn_if` (0x421BCF): `sound::race_turn` while the race's sounds are on.
//!
//! In:  nothing
//! Out: ebx 0x5317707 when it called (a leftover constant); everything else preserved
//! Only with `dword_6218F0` not 0. One caller, `sound::race_sound_tick`.
//! Ungated: `race_turn`'s quit jump never comes back.
//! Shadow mode runs the original alone (sounds are sent).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::sound::turn_parts::{Part, turn_entry};

/// dword_6218F0: the race's sounds on.
const ON: usize = 0x6218F0;
pub const RACE_TURN: u32 = 0x40D5F3;

// ---- Rust version ----

pub fn race_turn_if_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if unsafe { game_read::<u32>(rva(ON)) } == 0 {
        return Ok(());
    }
    part(RACE_TURN, r)?;
    r.ebx = 0x531_7707;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, race_turn_if_in)
}
