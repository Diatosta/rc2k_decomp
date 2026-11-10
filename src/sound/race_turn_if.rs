//! `sound::race_turn_if` (0x421BCF): `sound::race_turn` while the race's sounds are on.
//!
//! Only with `dword_6218F0` not 0. One caller, `sound::race_sound_tick`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

/// `sound::race_turn` while the race's sounds are on: true when it ran.
pub fn race_turn_if(
    sound: &crate::sound::owner::Sound,
    input: &crate::input::owner::Input,
) -> Result<bool, Refused> {
    let audio = crate::audio::owner::audio();
    if sound.race.sounds_on.get() == 0 {
        return Ok(false);
    }
    crate::sound::race_turn::race_turn(sound, input, audio)?;
    Ok(true)
}
