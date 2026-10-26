//! `audio::take_commands` (0x40A8A4): the audio thread takes the commands the game queued.
//!
//! In:  ebx, ecx, edx, edi, ebp = carried into the commands (see below)
//! Out: every register preserved
//!
//! While `g_soundCommandCount` is not 0: the record at `g_soundCommandRead` (0x20 bytes in
//! `g_soundCommands`: the number, then the sender's eax, ebx, ecx, edx, esi, edi, ebp; see
//! `sound/queue.rs`) in esi, its number in eax, and: 1 `audio::start_voice` (eax its eax as
//! a signed word); 2 `g_eaxRoom` = eax; 3 `g_soundListLoaded` = eax, `audio::load_sounds`;
//! 4 `audio::free_voices`, `g_soundListLoaded` = 0; 5 `audio::set_mode` (eax); 6
//! `audio::command_6` (nothing); 7 `audio::stop_all_voices`; 8 `audio::volume_command`
//! (eax); 9 `audio::stream_command` (eax, ecx, edx); 0xB `audio::music_command` (eax, ebx,
//! ecx, edx); any other nothing. Then the count down (atomically: the senders add to it from
//! other threads), the read place on (0 at 0x400); 0xA ends the turn's taking. The
//! registers carry over from one command to the next: what 9 and 0xB load stays for the
//! next ones, and `set_mode` hands ebx, ecx and edx to `audio::stop_streams` (edx its
//! action).
//! `set_mode` is called by the game's address (not ported). Refuses a read place past the
//! queue.

use crate::audio::store::{AUDIO, COMMANDS};
use crate::errors::{BadInput, Refused};
use std::sync::atomic::Ordering;

/// Converted: `rc2k_core::audio::commands::decode` and `next_place`. Plain since
/// 2026-10-23 (phase 3): each command's call gets what it reads (`decode`); the registers
/// the original carries from one command to the next reached only `set_mode`'s
/// `stop_streams`, which stops with action 0 (section 16).
pub fn take_commands() -> Result<(), Refused> {
    use crate::audio::stream_commands::{music_command, stream_command};
    use rc2k_core::audio::commands::{Command, decode, next_place};
    loop {
        if AUDIO.command_count.get() == 0 {
            return Ok(());
        }
        let place = AUDIO.command_read.get();
        let Some(command) = COMMANDS.records.get(place as usize) else {
            return Err(BadInput("a command read place past the queue").into());
        };
        let record = command.get();
        let command = decode(&record);
        match command {
            Command::Start(sound) => crate::audio::start_voice::start_voice(sound, &record)?,
            Command::Room(room) => AUDIO.eax_room.set(room),
            Command::Load(list) => {
                AUDIO.sound_list_loaded.set(list);
                crate::audio::load_sounds::load_sounds(list)?;
            }
            Command::Free => {
                crate::audio::free_voices::free_voices()?;
                AUDIO.sound_list_loaded.set(0);
            }
            Command::Mode(mode) => {
                crate::audio::set_mode::set_mode(mode)?;
            }
            Command::Six => {}
            Command::StopAll => crate::audio::stop_all_voices::stop_all_voices()?,
            Command::Volume(volume) => {
                crate::audio::volume_command::volume_command(volume, &mut |v| {
                    crate::audio::set_master_volume::set_master_volume_in_game(v)
                })
            }
            Command::Stream {
                path,
                volume,
                action,
            } => stream_command(path, volume as i32, action)?,
            Command::Music {
                tune,
                next,
                volume,
                action,
            } => music_command(tune, next, volume as i32, action)?,
            Command::EndOfTurn | Command::Other(_) => {}
        }
        AUDIO.command_count.atomic().fetch_sub(1, Ordering::SeqCst);
        AUDIO.command_read.set(next_place(place));
        if command == Command::EndOfTurn {
            return Ok(());
        }
    }
}
