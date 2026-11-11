//! `sound::play_sound` (0x409B36): sends the audio thread a sound to play (command 1).
//!
//! Takes: sound, volume, ;, the sound's place when the mode is 3D
//! Nothing while sounds are held back (`dword_506C84` is 1; `sound::hold_sounds_back` sets it,
//! `sound::let_sounds_play` clears it). Unless the sound mode in use (`dword_4F82BC`) has bit
//! 0x100000 (the 3D modes), the command goes with esi, edi and ebp = 0xFF800000; with
//! it, with esi = ecx and ecx = 0.
//! 14 callers.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::audio::owner::audio;
use crate::sound::queue::send_command;
use rc2k_core::audio::commands::{Command, Start};

// ---- game data ----

/// `sound` at `volume` and `rate`; `place` its ecx, edi and ebp (the pan, or in the 3D
/// modes the place). The audio thread reads every register of command 1.
pub fn play_sound(sound: u32, volume: u32, rate: u32, place: [u32; 3]) {
    let audio = audio();
    let (held_back, mode) = (
        audio.state.sounds_held_back.get(),
        audio.state.sound_mode.get(),
    );
    // Converted: `rc2k_core::sound::senders::sound_place`.
    if let Some([esi, ecx, edi, ebp]) =
        rc2k_core::sound::senders::sound_place(held_back, mode, place)
    {
        send_command(Command::Start(Start {
            sound_word: sound,
            volume,
            pan: ecx,
            frequency: rate,
            place: [esi, edi, ebp],
        }));
    }
}
