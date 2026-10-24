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
//! queue. The loop goes back into the patched bytes (0x40A8A5): a private copy.
//! Shadow mode runs the Rust version too (the original's count-down is not atomic: see
//! `dispatch`). Self-test: a redirected copy whose queue and
//! variables are the test's and whose calls go to stand-ins noting every register, 40 cases.

use crate::audio::store::Slot;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::store::addr32 as a;
use crate::store::game_addr;
use std::sync::atomic::{AtomicU32, Ordering};

pub const COUNT_RVA: Slot = Slot::at(0x506A1C); // g_soundCommandCount
pub const READ_RVA: Slot = Slot::at(0x506A20); // g_soundCommandRead
pub const LIST_LOADED_RVA: Slot = Slot::at(0x50695C); // g_soundListLoaded
#[derive(Clone, Copy)]
pub struct Vars {
    pub count: u32,
    pub read: u32,
    pub records: u32,
    pub eax_room: u32,
    pub list: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            count: a(COUNT_RVA),
            read: a(READ_RVA),
            records: a(crate::sound::queue::RECORDS_RVA),
            eax_room: a(EAX_ROOM_RVA),
            list: a(LIST_LOADED_RVA),
        }
    }
}

/// Converted: `rc2k_core::audio::commands::decode` and `next_place`. Plain since
/// 2026-10-23 (phase 3): each command's call gets what it reads (`decode`); the registers
/// the original carries from one command to the next reached only `set_mode`'s
/// `stop_streams`, which stops with action 0 (section 16).
pub fn take_commands(v: &Vars) -> Result<(), Refused> {
    use crate::audio::stream_commands::{MusicVars, music_command, stream_command};
    use rc2k_core::audio::commands::{Command, decode, next_place};
    unsafe {
        loop {
            if read(v.count) == 0 {
                return Ok(());
            }
            let place = read(v.read);
            if place >= crate::sound::queue::RECORDS {
                return Err(BadInput("a command read place past the queue").into());
            }
            let at = v.records + place * 0x20;
            let record = (at as *const [u32; 8]).read_unaligned();
            let command = decode(&record);
            match command {
                Command::Start(sound) => crate::audio::start_voice::start_voice(
                    &World::game(),
                    sound,
                    &crate::audio::start_voice::command(at),
                )?,
                Command::Room(room) => write(v.eax_room, room),
                Command::Load(list) => {
                    write(v.list, list);
                    crate::audio::load_sounds::load_sounds(
                        list,
                        &crate::audio::load_sounds::Vars::game(),
                    )?;
                }
                Command::Free => {
                    crate::audio::free_voices::free_voices(
                        &World::game(),
                        &crate::audio::free_voices::game_vars(),
                    )?;
                    write(v.list, 0);
                }
                Command::Mode(mode) => {
                    crate::audio::set_mode::set_mode(
                        mode,
                        &crate::audio::set_mode::ModeVars::game(),
                    )?;
                }
                Command::Six => {}
                Command::StopAll => {
                    let w = World::game();
                    crate::audio::stop_all_voices::stop_all_voices(
                        w.playing,
                        w.stopping,
                        w.channels_used,
                    )?
                }
                Command::Volume(volume) => crate::audio::volume_command::volume_command(
                    volume,
                    game_addr(MASTER_VOLUME_RVA) as u32,
                    &mut |v| crate::audio::set_master_volume::set_master_volume_in_game(v),
                ),
                Command::Stream {
                    path,
                    volume,
                    action,
                } => stream_command(
                    path,
                    volume as i32,
                    action,
                    game_addr(crate::audio::stream::SOUND_STREAM_RVA) as u32,
                )?,
                Command::Music {
                    tune,
                    next,
                    volume,
                    action,
                } => music_command(tune, next, volume as i32, action, &MusicVars::game())?,
                Command::EndOfTurn | Command::Other(_) => {}
            }
            (*(v.count as *const AtomicU32)).fetch_sub(1, Ordering::SeqCst);
            write(v.read, next_place(place));
            if command == Command::EndOfTurn {
                return Ok(());
            }
        }
    }
}
