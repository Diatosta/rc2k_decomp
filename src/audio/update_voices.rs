//! `audio::update_voices` (0x40AC77): the turn's update of the voices, every 20 ms.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! By the mode: with A3D (`g_useA3d` 1) the chunk at 0x40CB2A; with voices' own buffers
//! (`g_soundMode` bit 0x10000) the buffers' way, here; else with the game's mixer (bit
//! 0x40000) the chunk at 0x40BC05; else nothing. The two chunks are called at the game's
//! addresses (they begin with `pusha` and end with `popa; retn`).
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
//! Shadow mode runs the original alone. Self-test: a redirected copy (0x40AC77 .. 0x40AEB9,
//! with `audio::clear_channels` between) whose variables are the test's and whose eleven
//! calls note the voice, 200 states by lot.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

const MODE_MIXER: u32 = 0x4_0000;
const A3D_WAY: usize = 0x40CB2A;
const MIXER_WAY: usize = 0x40BC05;
const ENVIRONMENT_RVA: usize = rva(0x6D1DD4); // dword_6D1DD4
const ENVIRONMENT_KEPT_RVA: usize = rva(0x506C80); // dword_506C80

#[derive(Clone, Copy)]
pub struct Vars {
    pub a3d: u32,
    pub mode: u32,
    pub playing: u32,
    pub stopping: u32,
    pub used: u32,
    pub commit: u32,
    pub room_asked: u32,
    pub room_set: u32,
    pub environment_kept: u32,
    pub environment: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            a3d: a(A3D_RVA),
            mode: a(SOUND_MODE_RVA),
            playing: a(PLAYING_RVA),
            stopping: a(STOPPING_RVA),
            used: a(CHANNELS_USED_RVA),
            commit: a(COMMIT_RVA),
            room_asked: a(EAX_ROOM_RVA),
            room_set: a(EAX_ROOM_SET_RVA),
            environment_kept: a(ENVIRONMENT_KEPT_RVA),
            environment: a(ENVIRONMENT_RVA),
        }
    }
}

/// Converted: the own-buffers way is `rc2k_core::audio::voices::update_buffers`, each of
/// its steps a callee here (esi the voice); the A3D and mixer ways stay the game's chunks.
pub unsafe fn update_voices(v: &Vars) -> Result<(), Refused> {
    use rc2k_core::audio::voices::{Lists, Room, Step, update_buffers};
    unsafe {
        if read(v.a3d) == 1 {
            return crate::audio::update_voices::game_part(10, &mut Regs::default());
        }
        let mode = read(v.mode);
        if mode & crate::audio::make_buffer::MODE_OWN_BUFFERS == 0 {
            if mode & MODE_MIXER != 0 {
                crate::audio::update_voices::game_part(11, &mut Regs::default())?;
            }
            return Ok(());
        }
        let mut l = Lists {
            playing: read(v.playing),
            stopping: read(v.stopping),
            channels_used: read(v.used),
            channel_count: 0,
        };
        let mut room = Room {
            asked: read(v.room_asked),
            set: read(v.room_set),
            environment: read(v.environment),
            kept: read(v.environment_kept),
            commit: read(v.commit),
        };
        let store = |l: &Lists, room: &Room| {
            write(v.playing, l.playing);
            write(v.stopping, l.stopping);
            write(v.used, l.channels_used);
            write(v.room_set, room.set);
            write(v.environment_kept, room.kept);
            write(v.commit, room.commit);
        };
        let mut pool = GamePool::new(&World::game());
        // No step reads what the update changes (the room's step reads the room asked).
        let done = update_buffers::<_, Refused>(&mut pool, &mut l, mode, &mut room, |step| {
            let (which, at) = match step {
                Step::IsPlaying(at) => (0, at),
                Step::Room => (1, 0),
                Step::Commit => (2, 0),
                Step::Frequency(at) => (3, at),
                Step::Volume(at) => (4, at),
                Step::Pan(at) => (5, at),
                Step::Place(at) => (6, at),
                Step::Rewind(at) => (7, at),
                Step::Play(at) => (8, at),
                Step::Stop(at) => (9, at),
            };
            let mut r = Regs {
                esi: at,
                ..Regs::default()
            };
            crate::audio::update_voices::game_part(which, &mut r)?;
            Ok(which != 0 || !crate::audio::buffer_play::zero(&r))
        });
        store(&l, &room);
        done
    }
}

/// The game's callees: the Rust versions, the two other ways at the game's addresses.
pub fn game_part(which: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::audio::{buffer_play as play, buffer_settings as set};
    unsafe {
        let v = || voice(r.esi);
        match which {
            0 => {
                let status = play::is_playing(v())?;
                r.set_zero(status & play::PLAYING == 0);
            }
            1 => r.eax = crate::audio::set_eax_room::in_game()?,
            2 => {
                play::commit_3d(game_addr(play::LISTENER_3D_RVA) as u32)?;
            }
            3 => {
                set::set_buffer_frequency(v())?;
            }
            4 => {
                set::set_buffer_volume(v())?;
            }
            5 => {
                set::set_buffer_pan(v())?;
            }
            6 => {
                set::set_buffer_place(v())?;
            }
            7 => {
                play::rewind_buffer(v())?;
            }
            8 => {
                play::play_buffer_in_game(v())?;
            }
            9 => {
                play::stop_buffer(v())?;
            }
            10 => crate::calls::call(rva(A3D_WAY), r),
            _ => crate::calls::call(rva(MIXER_WAY), r),
        }
    }
    Ok(())
}
