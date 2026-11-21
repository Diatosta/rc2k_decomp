//! Closes everything of the sound (the audio thread's end): streams, voices, the way, the primary
//! buffer and the sound itself, then the sound mode is 0.

use crate::audio::own::AudioThread;
use crate::errors::Refused;

pub(super) fn close_all(
    audio: &AudioThread,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let state = &audio.state;
    use crate::audio::sound_device::*;
    crate::audio::stop_streams::stop_streams(audio)?;
    crate::audio::free_voices::free_voices(audio)?;
    crate::audio::mixer_set_up::close_way(audio, system, state.sound_mode.get())?;
    release_primary(state);
    close_sound(state, system)?;
    state.sound_mode.set(0);
    Ok(())
}
