//! The turn's update of the voices, every 20 ms. By the mode: with voices' own buffers the
//! buffers' way, here; with the game's mixer `audio::update_mixer_voices`; else nothing. A3D is
//! refused.
//!
//! The buffers' way, in three passes:
//! - the stopping list: a voice sounding is stopped; every voice's state 0, its channel freed, and
//!   the list emptied;
//! - the playing list: a voice with dirty bits and a buffer gets, as marked, its frequency, volume,
//!   pan and in 3D its place; one not sounding, or marked for a restart, is rewound and, if not
//!   sounding, played; its dirty bits 0. A voice that does not loop whose buffer no longer plays
//!   has its channel freed and goes to the stopping list's front;
//! - the stopping list again (so a voice that just ended is stopped the same turn).
//!
//! Then in 3D: with EAX the room is set when the room asked is not the one set, or the
//! environment is not the one kept; and the 3D settings are committed when marked.
//!
//! Refuses, before anything: a list that does not end within 0x200 voices; a playing voice that
//! does not loop without its channel or buffer; a voice to stop without its buffer; a marked
//! voice's volume or pan past the volume table.

use crate::audio::own::AudioThread;
use crate::audio::voices::{GamePool, voice_of};
use crate::errors::{BadInput, Refused};

const MODE_MIXER: u32 = 0x4_0000;

pub fn update_voices(audio: &AudioThread) -> Result<(), Refused> {
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
        environment: audio.environment.get(),
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
                play::play_buffer(&audio.state, &voice_of(&audio.state, at))?;
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
