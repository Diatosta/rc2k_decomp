//! `sound::hold_sounds_back`: holds the sounds back (the flag is 1).
//!
//! Its twin `sound::let_sounds_play` sets it to 0; `sound::play_sound` plays nothing while it is 1.
//! One caller.

pub fn hold_sounds_back(audio: &crate::audio::owner::Audio) {
    audio.state.sounds_held_back.set(1);
}
