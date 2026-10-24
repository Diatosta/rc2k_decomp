//! `input::begin_effects` (0x40F29D): the race's force feedback begun: if any sound object
//! is in use (`g_raceSoundObjectCount`, a race is on) both records' effects asked to start
//! (`input::start_effects`); the clock (`system::tick_count`) kept as the effects' last
//! step (`g_effectsLastTime`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 3, the race's
//! start).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::effects::LAST_TIME;
use crate::sound::turn_parts::COUNT;

/// The effects started with sound objects in use (`g_raceSoundObjectCount`), and the
/// effects' clock set.
pub fn begin_effects() -> Result<(), Refused> {
    if unsafe { crate::store::game_read::<u32>(COUNT) } != 0 {
        crate::input::start_effects::start_effects()?;
    }
    unsafe { crate::store::game_write(LAST_TIME, crate::system::tick_count::tick_count()) };
    Ok(())
}
