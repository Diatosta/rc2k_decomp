//! `audio::update_voices` (0x40AC77): the turn's update of the voices, every 20 ms.
//!
//! By the mode: with A3D (`g_useA3d` 1) the chunk at 0x40CB2A; with voices' own buffers
//! (`g_soundMode` bit 0x10000) the buffers' way, here; else with the game's mixer (bit
//! 0x40000) the chunk at 0x40BC05; else nothing. The two chunks are called at the game's
//! addresses (they begin with `pusha` and end with `popa; retn`). Plain since 2026-10-23
//! (phase 3): the mixer's way is `audio::update_mixer_voices`; A3D is refused.
//! The buffers' way, in three passes:
//! - the stopping list: a voice sounding (state 2) stopped (`audio::stop_buffer`); every
//!   voice's state 0, its channel freed and 0, and each link of the list 0 (it is emptied);
//! - the playing list: a voice with dirty bits and a buffer gets, as marked, its
//!   frequency (4), volume (1), pan (2) and in 3D, with a 3D buffer, its place (8, and
//!   `g_commit3d` 1); one not sounding, or marked for a restart (0x20), is rewound
//!   (`audio::rewind_buffer`) and, if not sounding, played (`audio::play_buffer`, state
//!   2); its dirty bits 0. A voice that does not loop (kind other than 1) and whose buffer
//!   no longer plays (`audio::is_playing`) has its channel freed and 0, goes to the
//!   stopping list's front, and `g_channelsUsed` is one less;
//! - the stopping list again (so a voice that just ended is stopped the same turn).
//!
//! Then in 3D: with EAX (bit 0x1000000) the room is set (`audio::set_eax_room`) when the
//! room asked (`g_eaxRoom`) is not the one set, or `dword_6D1DD4` is not the one kept
//! (`dword_506C80`), each then kept; and with `g_commit3d` 1, `audio::commit_3d` and the flag 0.
//! Refuses, before anything: a list that does not end within 0x200 voices; a playing voice
//! that does not loop without its channel or buffer; a voice to stop without its buffer; a
//! marked voice's volume or pan past the volume table.

use crate::audio::owner::Audio;
use crate::audio::store::ENVIRONMENT;
use crate::audio::voices::{GamePool, voice_of};
use crate::errors::{BadInput, Refused};

const MODE_MIXER: u32 = 0x4_0000;

/// Converted: the own-buffers way is `rc2k_core::audio::voices::update_buffers`, each of
/// its steps a callee here (esi the voice); the A3D and mixer ways stay the game's chunks.
pub fn update_voices(audio: &Audio) -> Result<(), Refused> {
    use rc2k_core::audio::voices::{Lists, Room, Step, update_buffers};
    if audio.state.use_a3d.get() == 1 {
        return Err(BadInput("A3D (the chunk at 0x40CB2A)").into());
    }
    let mode = audio.state.sound_mode.get();
    if mode & crate::audio::make_buffer::MODE_OWN_BUFFERS == 0 {
        if mode & MODE_MIXER != 0 {
            crate::audio::update_mixer_voices::update_mixer_voices_in_game(audio)?;
        }
        return Ok(());
    }
    let mut l = Lists {
        playing: audio.state.playing_voices.get(),
        stopping: audio.state.stopping_voices.get(),
        channels_used: audio.state.channels_used.get(),
        channel_count: 0,
    };
    let mut room = Room {
        asked: audio.state.eax_room.get(),
        set: audio.state.eax_room_set.get(),
        environment: ENVIRONMENT.first().get(),
        kept: audio.state.environment_kept.get(),
        commit: audio.state.commit_3d.get(),
    };
    let store = |l: &Lists, room: &Room| {
        audio.state.playing_voices.set(l.playing);
        audio.state.stopping_voices.set(l.stopping);
        audio.state.channels_used.set(l.channels_used);
        audio.state.eax_room_set.set(room.set);
        audio.state.environment_kept.set(room.kept);
        audio.state.commit_3d.set(room.commit);
    };
    let mut pool = GamePool::new(audio);
    // No step reads what the update changes (the room's step reads the room asked).
    let done = update_buffers::<_, Refused>(&mut pool, &mut l, mode, &mut room, |step| {
        use crate::audio::{buffer_play as play, buffer_settings as set};
        match step {
            Step::IsPlaying(at) => {
                return Ok(play::is_playing(&audio.state, &voice_of(&audio.state, at))?
                    & play::PLAYING
                    != 0);
            }
            Step::Room => {
                crate::audio::set_eax_room::in_game(&audio.state)?;
            }
            Step::Commit => {
                play::commit_3d(&audio.state)?;
            }
            Step::Frequency(at) => {
                set::set_buffer_frequency(&audio.state, &voice_of(&audio.state, at))?;
            }
            Step::Volume(at) => {
                set::set_buffer_volume(&audio.state, &voice_of(&audio.state, at))?;
            }
            Step::Pan(at) => {
                set::set_buffer_pan(&audio.state, &voice_of(&audio.state, at))?;
            }
            Step::Place(at) => {
                set::set_buffer_place(&voice_of(&audio.state, at))?;
            }
            Step::Rewind(at) => {
                play::rewind_buffer(&audio.state, &voice_of(&audio.state, at))?;
            }
            Step::Play(at) => {
                play::play_buffer_in_game(&audio.state, &voice_of(&audio.state, at))?;
            }
            Step::Stop(at) => {
                play::stop_buffer(&audio.state, &voice_of(&audio.state, at))?;
            }
        }

        Ok(true)
    });
    store(&l, &room);
    done
}
