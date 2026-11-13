//! `audio::free_voices` (0x40AF4B): command 4: every sound's voices and buffers given back.
//!
//! If voices are in use (`g_voiceCount`): `audio::release_eax`, `audio::release_voice_manager`,
//! `audio::release_voice` on each of them, `g_voiceCount` 0. Then every sample record's
//! `first_voice` 0, `audio::clear_channels`, `g_soundBufferLost`, `g_commit3d`,
//! `g_channelsUsed` and both lists 0, and the samples' memory given back (0x409E0F,
//! `memory::free_list` on `g_sampleMemory`, inlined). Callers: `audio::close_all` (the thread's
//! end), `audio::set_mode`, `audio::take_commands`.
//! Refuses more than 0x200 voices in use.

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
        // The voices by their places, as the original steps over them.
        for n in 0..count as usize {
            crate::audio::release_voice::release_voice_in_game(
                &audio.state,
                crate::audio::voices::voice_id(n as u32),
            );
        }
        audio.state.voice_count.set(0);
    }
    for n in 0..SAMPLES as usize {
        audio.sample_records.records[n].update(|r| r.first_voice = 0);
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
