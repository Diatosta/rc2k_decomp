//! `sound::stream_playing`: whether the music stream's playing flag (`AudioShared.sound_stream`) is
//! 1 (first result) or below 1 (second result).
//!
//! Called by `menu::championship_welcome` and one more; `sound::step_car_stream` has it inlined.

use crate::audio::owner::Audio;

/// (playing is 1, playing is below 1).
pub fn stream_playing(audio: &Audio) -> (bool, bool) {
    let playing = audio.state.sound_stream.get().playing;
    (playing == 1, playing < 1)
}
