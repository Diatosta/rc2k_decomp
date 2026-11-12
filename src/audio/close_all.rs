//! `audio::close_all` (0x409DD5): everything of the sound closed (the audio thread's end).
//!
//! Takes: handed to `audio::stop_streams` (its action)
//!
//! `audio::stop_streams`, `audio::free_voices`, `audio::close_way`,
//! `audio::release_primary`, `audio::close_sound`, then `g_soundMode` 0. One caller, the
//! audio thread.

use crate::audio::owner::Audio;
use crate::errors::Refused;

pub fn close_all(
    audio: &Audio,
    system: &crate::system::owner::System,
    tune_turn: &crate::session::Dword,
) -> Result<(), Refused> {
    let state = &audio.state;
    use crate::audio::sound_device::*;
    crate::audio::stop_streams::stop_streams(audio, tune_turn)?;
    crate::audio::free_voices::free_voices(audio)?;
    crate::audio::mixer_set_up::close_way(audio, system, state.sound_mode.get())?;
    release_primary(state);
    close_sound(state, system)?;
    state.sound_mode.set(0);
    Ok(())
}
