//! `sound::let_sounds_play` (0x409B76): `g_soundsHeldBack` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Its twin `sound::hold_sounds_back` (0x409B6B) sets it to 1; `sound::play_sound` plays
//! nothing while it is 1. Five callers (`race::start_race_sounds` and four more).

use crate::audio::owner::audio;

pub fn let_sounds_play() {
    audio().state.sounds_held_back.set(0);
}
