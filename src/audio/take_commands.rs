//! The audio thread takes the commands the game queued (`sound/queue.rs`), oldest first: 1 start
//! a voice; 2 set the EAX room; 3 load a sound list; 4 free the voices; 5 set the mode; 7 stop all
//! voices; 8 the master volume; 9 the stream command; 0xB the music command; 0xA ends the turn's
//! taking. A command is dropped from the queue when done; the queue counts it as waiting until
//! then.

use crate::audio::own::AudioThread;
use crate::errors::Refused;

pub fn take_commands(
    audio: &AudioThread,
    system: &'static crate::system::owner::System,
    sets: &crate::sound::sets_store::SoundSets,
    menu_sounds: &crate::sound::sets_store::MenuSounds,
) -> Result<(), Refused> {
    use crate::audio::stream_commands::{music_command, stream_command};
    use rc2k_core::audio::commands::Command;
    loop {
        let Some(command) = crate::sound::queue::locked(audio).front() else {
            return Ok(());
        };
        match command {
            Command::Start(start) => {
                crate::audio::start_voice::start_voice(audio, start.sound as i32, &start)?;
            }
            Command::Room(room) => {
                audio.state.eax_room.set(room);
            }
            Command::Load(list) => {
                audio.state.sound_list_loaded.set(list);
                crate::audio::load_sounds::load_sounds(audio, system, sets, menu_sounds, list)?;
            }
            Command::Free => {
                crate::audio::free_voices::free_voices(audio)?;
                audio.state.sound_list_loaded.set(None);
            }
            Command::Mode(mode) => {
                crate::audio::set_mode::set_mode(audio, system, mode)?;
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
        crate::sound::queue::locked(audio).pop();
        if command == Command::EndOfTurn {
            #[cfg(feature = "remote")]
            crate::audio::mix_hash::turn_end_taken();
            return Ok(());
        }
    }
}
