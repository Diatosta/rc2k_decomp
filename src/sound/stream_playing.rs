//! `sound::stream_playing` (0x40C4D5): the music stream's playing flag (`AudioState.sound_stream`,
//! `g_soundStream.playing`) compared with 1: zero set when it plays, carry when it is 0.
//!
//! Callers: 0x46F7AD (`jz`), `menu::championship_welcome` (`jnz`); `sound::step_car_stream` has it
//! inlined.

use crate::audio::owner::Audio;

/// (zero, carry) as the original's `cmp` leaves them.
pub fn stream_playing(audio: &Audio) -> (bool, bool) {
    let playing = audio.state.sound_stream.get().playing;
    (playing == 1, playing < 1)
}
