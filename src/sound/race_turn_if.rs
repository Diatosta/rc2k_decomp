//! `sound::race_turn_if` (0x421BCF): `sound::race_turn` while the race's sounds are on.
//!
//! In:  nothing
//! Out: ebx 0x5317707 when it called (a leftover constant); everything else preserved
//! Only with `dword_6218F0` not 0. One caller, `sound::race_sound_tick`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// `sound::race_turn` while the race's sounds are on: true when it ran.
pub fn race_turn_if() -> Result<bool, Refused> {
    if crate::sound::store::RACE_SOUNDS_ON.get() == 0 {
        return Ok(false);
    }
    crate::sound::race_turn::race_turn()?;
    Ok(true)
}
