//! `sound::race_turn_if` (0x421BCF): `sound::race_turn` while the race's sounds are on.
//!
//! Only with `dword_6218F0` not 0. One caller, `sound::race_sound_tick`.

use crate::errors::Refused;

/// `sound::race_turn` while the race's sounds are on: true when it ran.
pub fn race_turn_if(
    sound: &crate::sound::owner::Sound,
    input: &crate::input::owner::Input,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
    race: &crate::race::owner::Race,
) -> Result<bool, Refused> {
    if sound.race.sounds_on.get().get() == 0 {
        return Ok(false);
    }
    crate::sound::race_turn::race_turn(sound, input, audio, system, race)?;
    Ok(true)
}
