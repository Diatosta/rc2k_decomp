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

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use std::sync::atomic::{AtomicU32, Ordering};

pub const COUNT_RVA: usize = rva(0x506A1C); // g_soundCommandCount
pub const READ_RVA: usize = rva(0x506A20); // g_soundCommandRead
pub const LIST_LOADED_RVA: usize = rva(0x50695C); // g_soundListLoaded
const SET_MODE: usize = 0x40A426; // audio::set_mode
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
        let a = |r| game_addr(r) as u32;
        Self {
            count: a(COUNT_RVA),
            read: a(READ_RVA),
            records: a(crate::sound::queue::RECORDS_RVA),
            eax_room: a(EAX_ROOM_RVA),
            list: a(LIST_LOADED_RVA),
        }
    }
}

/// Converted: `rc2k_core::audio::commands::decode` and `next_place`; the registers carry
/// over from one command to the next as the original's do.
pub fn take_commands(entry: &Regs, v: &Vars) -> Result<(), Refused> {
    use rc2k_core::audio::commands::{Command, decode, next_place};
    let mut r = *entry;
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
            r.esi = at;
            let record = (at as *const [u32; 8]).read_unaligned();
            r.eax = record[0];
            let command = decode(&record);
            match command {
                Command::Start(sound) => {
                    r.eax = sound as u32;
                    crate::audio::take_commands::game_part(0, &mut r)?;
                }
                Command::Room(room) => {
                    r.eax = room;
                    write(v.eax_room, room);
                }
                Command::Load(list) => {
                    r.eax = list;
                    write(v.list, list);
                    crate::audio::take_commands::game_part(1, &mut r)?;
                }
                Command::Free => {
                    crate::audio::take_commands::game_part(2, &mut r)?;
                    write(v.list, 0);
                }
                Command::Mode(mode) => {
                    r.eax = mode;
                    crate::audio::take_commands::game_part(3, &mut r)?;
                }
                Command::Six => crate::audio::take_commands::game_part(4, &mut r)?,
                Command::StopAll => crate::audio::take_commands::game_part(5, &mut r)?,
                Command::Volume(volume) => {
                    r.eax = volume;
                    crate::audio::take_commands::game_part(6, &mut r)?;
                }
                Command::Stream {
                    path,
                    volume,
                    action,
                } => {
                    (r.eax, r.ecx, r.edx) = (path, volume, action);
                    crate::audio::take_commands::game_part(7, &mut r)?;
                }
                Command::Music {
                    tune,
                    next,
                    volume,
                    action,
                } => {
                    (r.eax, r.ebx, r.ecx, r.edx) = (tune, next, volume, action);
                    crate::audio::take_commands::game_part(8, &mut r)?;
                }
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

/// The Rust versions, as the game has them; `set_mode` the game's own.
pub fn game_part(which: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::audio::stream_commands::{MusicVars, music_command, stream_command};
    unsafe {
        match which {
            0 => crate::audio::start_voice::start_voice(
                &World::game(),
                r.eax as i32,
                &crate::audio::start_voice::command(r.esi),
            )?,
            1 => crate::audio::load_sounds::load_sounds(
                r.eax,
                &crate::audio::load_sounds::Vars::game(),
                &mut crate::audio::load_sounds::game_part,
            )?,
            2 => crate::audio::free_voices::free_voices(
                &World::game(),
                &crate::audio::free_voices::game_vars(),
            )?,
            3 => {
                let mut c = *r;
                crate::calls::call(rva(SET_MODE), &mut c);
            }
            4 => {}
            5 => {
                let w = World::game();
                crate::audio::stop_all_voices::stop_all_voices(
                    w.playing,
                    w.stopping,
                    w.channels_used,
                )?
            }
            6 => crate::audio::volume_command::volume_command(
                r.eax,
                game_addr(MASTER_VOLUME_RVA) as u32,
                &mut |v| crate::audio::set_master_volume::set_master_volume_in_game(v),
            ),
            7 => stream_command(
                r.eax,
                r.ecx as i32,
                r.edx,
                game_addr(crate::audio::stream::SOUND_STREAM_RVA) as u32,
                &mut crate::audio::stream_commands::game_part,
            )?,
            _ => music_command(
                r.eax,
                r.ebx,
                r.ecx as i32,
                r.edx,
                &MusicVars::game(),
                &mut crate::audio::stream_commands::game_part,
            )?,
        }
    }
    Ok(())
}
