//! `audio::free_voices` (0x40AF4B): command 4: every sound's voices and buffers given back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! If voices are in use (`g_voiceCount`): `audio::release_eax`, `audio::release_voice_manager`,
//! `audio::release_voice` on each of them, `g_voiceCount` 0. Then every sample record's
//! `first_voice` 0, `audio::clear_channels`, `g_soundBufferLost`, `g_commit3d`,
//! `g_channelsUsed` and both lists 0, and the samples' memory given back (`sub_409E0F`,
//! `memory::free_list` on `g_sampleMemory`, inlined). Callers: `sub_409DD5` (the thread's
//! end), `audio::set_mode`, `audio::take_commands`.
//! Refuses more than 0x200 voices in use.

use crate::audio::store::{AUDIO, SAMPLE_RECORDS};
use crate::audio::voices::{SAMPLES, VOICES};
use crate::errors::{BadInput, Refused};

// ---- Rust version ----

pub fn free_voices() -> Result<(), Refused> {
    let count = AUDIO.voice_count.get();
    if count > VOICES {
        return Err(BadInput("more than 0x200 voices").into());
    }
    if count != 0 {
        crate::audio::release_eax::release_eax();
        crate::audio::release_voice_manager::release_voice_manager();
        // The voices by their places, as the original steps over them.
        for n in 0..count as usize {
            unsafe {
                crate::audio::release_voice::release_voice_in_game(AUDIO.voices[n].addr() as u32)
            };
        }
        AUDIO.voice_count.set(0);
    }
    for n in 0..SAMPLES as usize {
        SAMPLE_RECORDS.records[n].update(|r| r.first_voice = 0);
    }
    crate::audio::clear_channels::clear_channels();
    AUDIO.sound_buffer_lost.set(0);
    AUDIO.commit_3d.set(0);
    AUDIO.channels_used.set(0);
    AUDIO.playing_voices.set(0);
    AUDIO.stopping_voices.set(0);
    unsafe { crate::memory::free_list::free_list(AUDIO.sample_memory.as_ptr())? };
    Ok(())
}
