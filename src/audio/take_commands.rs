//! `audio::take_commands` (0x40A8A4): the audio thread takes the commands the game queued.
//!
//! While commands wait (`sound/queue.rs`), the oldest: 1 `audio::start_voice`; 2 `g_eaxRoom`
//! = the room; 3 `g_soundListLoaded` = the list's word, `audio::load_sounds`; 4
//! `audio::free_voices`, `g_soundListLoaded` = 0; 5 `audio::set_mode`; 7
//! `audio::stop_all_voices`; 8 `audio::volume_command`; 9 `audio::stream_command`; 0xB
//! `audio::music_command`. Then it is dropped from the queue, which counts it as waiting
//! until then; 0xA ends the turn's taking.
//! `set_mode` hands the next stream stop the action 0 (section 16).

use crate::audio::owner::Audio;
use crate::errors::Refused;

pub fn take_commands(audio: &Audio) -> Result<(), Refused> {
    use crate::audio::stream_commands::{music_command, stream_command};
    use rc2k_core::audio::commands::{Command, SoundList};
    loop {
        let Some(command) = crate::sound::queue::locked().front() else {
            return Ok(());
        };
        match command {
            Command::Start(start) => {
                crate::audio::start_voice::start_voice(audio, start.sound(), &start)?;
            }
            Command::Room(room) => audio.state.eax_room.set(room),
            Command::Load(list) => {
                audio
                    .state
                    .sound_list_loaded
                    .set(list.map_or(0, SoundList::word));
                crate::audio::load_sounds::load_sounds(audio, list)?;
            }
            Command::Free => {
                crate::audio::free_voices::free_voices(audio)?;
                audio.state.sound_list_loaded.set(0);
            }
            Command::Mode(mode) => {
                crate::audio::set_mode::set_mode(audio, mode)?;
            }
            Command::StopAll => crate::audio::stop_all_voices::stop_all_voices(audio)?,
            Command::Volume(volume) => {
                crate::audio::volume_command::volume_command(&audio.state, volume, &mut |v| {
                    crate::audio::set_master_volume::set_master_volume(&audio.state, v);
                });
            }
            Command::Stream {
                path,
                volume,
                action,
            } => stream_command(audio, path, volume as i32, action)?,
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => music_command(&audio.state, tune, next, volume as i32, action)?,
            Command::EndOfTurn => {}
        }
        crate::sound::queue::locked().pop();
        if command == Command::EndOfTurn {
            return Ok(());
        }
    }
}
