//! `sound::let_sounds_play`: lets the sounds play (the held-back flag is 0).
//!
//! Its twin `sound::hold_sounds_back` sets it to 1; `sound::play_sound` plays nothing while it is
//! 1.

use crate::audio::owner::Audio;

pub fn let_sounds_play_in(audio: &Audio) {
    audio.state.sounds_held_back.set(0);
}
