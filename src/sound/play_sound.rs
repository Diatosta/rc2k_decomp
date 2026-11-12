//! `sound::play_sound` (0x409B36): sends the audio thread a sound to play (command 1).
//!
//! Nothing while sounds are held back (`AudioState.sounds_held_back` is 1; `sound::hold_sounds_back` sets it,
//! `sound::let_sounds_play` clears it). Unless the sound mode in use (`AudioState.sound_mode`) has bit
//! 0x100000 (the 3D modes), the command carries no place (the three place words are
//! 0xFF800000) and the caller's first place value is the pan; in the 3D modes the three
//! place values are the sound's place and the pan is 0.
//! 14 callers.

use crate::audio::owner::Audio;
use crate::sound::queue::send_command_in;
use rc2k_core::audio::commands::{Command, Start};

// ---- game data ----

/// `sound` at `volume` and `rate`; `place` is the pan in its first value, or in the 3D
/// modes the sound's place (three floats' bits).
pub fn play_sound_in(audio: &Audio, sound: u32, volume: u32, rate: u32, place: [u32; 3]) {
    let (held_back, mode) = (
        audio.state.sounds_held_back.get(),
        audio.state.sound_mode.get(),
    );
    // Converted: `rc2k_core::sound::senders::sound_place`.
    if let Some([place_a, pan, place_b, place_c]) =
        rc2k_core::sound::senders::sound_place(held_back, mode, place)
    {
        send_command_in(
            audio,
            Command::Start(Start {
                sound_word: sound,
                volume,
                pan,
                frequency: rate,
                place: [place_a, place_b, place_c],
            }),
        );
    }
}
