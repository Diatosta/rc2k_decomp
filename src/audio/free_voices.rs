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

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::store::game_addr;

// ---- Rust version ----

/// Where its variables are: `World` and these.
#[derive(Clone, Copy)]
pub struct Vars {
    pub voice_count: u32,
    pub voices: u32,
    pub lost: u32,
    pub commit: u32,
}

pub unsafe fn free_voices(w: &World, v: &Vars) -> Result<(), Refused> {
    unsafe {
        let count = read(v.voice_count);
        if count > VOICES {
            return Err(BadInput("more than 0x200 voices").into());
        }
        if count != 0 {
            crate::audio::release_eax::release_eax(crate::audio::release_eax::game_vars());
            crate::audio::release_voice_manager::release_voice_manager(
                game_addr(VOICE_MANAGER_RVA) as u32,
            );
            for n in 0..count {
                crate::audio::release_voice::release_voice_in_game(
                    v.voices + n * std::mem::size_of::<Voice>() as u32,
                );
            }
            write(v.voice_count, 0);
        }
        for n in 0..SAMPLES {
            sample(w, n).first_voice = 0;
        }
        crate::audio::clear_channels::clear_channels(World::game().channels);
        write(v.lost, 0);
        write(v.commit, 0);
        write(w.channels_used, 0);
        write(w.playing, 0);
        write(w.stopping, 0);
        crate::memory::free_list::free_list(game_addr(SAMPLE_MEMORY_RVA) as *mut _)?;
        Ok(())
    }
}

pub fn game_vars() -> Vars {
    Vars {
        voice_count: game_addr(VOICE_COUNT_RVA) as u32,
        voices: game_addr(VOICES_RVA) as u32,
        lost: game_addr(BUFFER_LOST_RVA) as u32,
        commit: game_addr(COMMIT_RVA) as u32,
    }
}
