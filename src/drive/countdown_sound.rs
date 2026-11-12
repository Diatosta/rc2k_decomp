//! `drive::countdown_sound` (0x40EC9C): for a listed car, plays the count down's sound on its
//! channel.
//!
//! Takes: the car's sound slot, the sound
//!
//! Caller: `drive::countdown_cue`.

// Converted: `rc2k_core::drive::countdown::beep_sound`.
pub fn countdown_sound(slot: u32, sound: u32, audio: &crate::audio::owner::Audio) {
    let Some(beep) = rc2k_core::drive::countdown::beep_sound(slot, sound) else {
        return;
    };
    crate::sound::play_sound::play_sound_in(audio, beep.sound, beep.volume, beep.volume, [0; 3]);
}
