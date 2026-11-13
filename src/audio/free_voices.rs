//! Command 4: gives back every sound's voices and buffers and the samples' memory, and resets the
//! channels and lists. Refuses more than 0x200 voices in use.

use crate::audio::own::AudioThread;
use crate::audio::voices::{SAMPLES, VOICES};
use crate::errors::{BadInput, Refused};

pub fn free_voices(audio: &AudioThread) -> Result<(), Refused> {
    let count = audio.state.voice_count.get();
    if count > VOICES {
        return Err(BadInput("more than 0x200 voices").into());
    }
    if count != 0 {
        crate::audio::release_eax::release_eax(&audio.state);
        crate::audio::release_voice_manager::release_voice_manager(&audio.state);
        for n in 0..count as usize {
            crate::audio::release_voice::release_voice_in_game(
                &audio.state,
                crate::audio::voices::voice_id(n as u32),
            );
        }
        audio.state.voice_count.set(0);
    }
    for n in 0..SAMPLES as usize {
        audio.sample_records.set_first_voice(n, 0);
    }
    crate::audio::clear_channels::clear_channels(audio);
    audio.state.sound_buffer_lost.set(0);
    audio.state.commit_3d.set(0);
    audio.state.channels_used.set(0);
    audio.state.playing_voices.set(0);
    audio.state.stopping_voices.set(0);
    crate::memory::free_list::free_list(&audio.state.sample_memory)?;
    // The kept samples with it (and every data id: no voice is left).
    crate::audio::sample_data::forget_all();
    Ok(())
}
