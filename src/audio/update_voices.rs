//! `audio::update_voices` (0x40AC77): the turn's update of the voices, every 20 ms.
//!
//! In:  nothing
//! Out: every register preserved
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
//! Shadow mode runs the original alone. Self-test: a redirected copy (0x40AC77 .. 0x40AEB9,
//! with `audio::clear_channels` between) whose variables are the test's and whose eleven
//! calls note the voice, 200 states by lot.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::store::Slot;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::store::addr32 as a;
use crate::store::game_addr;

const MODE_MIXER: u32 = 0x4_0000;
const ENVIRONMENT_KEPT_RVA: Slot = Slot::at(0x506C80); // dword_506C80

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
            environment: crate::audio::store::ENVIRONMENT.addr() as u32,
        }
    }
}

/// Converted: the own-buffers way is `rc2k_core::audio::voices::update_buffers`, each of
/// its steps a callee here (esi the voice); the A3D and mixer ways stay the game's chunks.
pub unsafe fn update_voices(v: &Vars) -> Result<(), Refused> {
    use rc2k_core::audio::voices::{Lists, Room, Step, update_buffers};
    unsafe {
        if read(v.a3d) == 1 {
            return Err(BadInput("A3D (the chunk at 0x40CB2A)").into());
        }
        let mode = read(v.mode);
        if mode & crate::audio::make_buffer::MODE_OWN_BUFFERS == 0 {
            if mode & MODE_MIXER != 0 {
                crate::audio::update_mixer_voices::update_mixer_voices_in_game()?;
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
            use crate::audio::{buffer_play as play, buffer_settings as set};
            match step {
                Step::IsPlaying(at) => return Ok(play::is_playing(voice(at))? & play::PLAYING != 0),
                Step::Room => {
                    crate::audio::set_eax_room::in_game()?;
                }
                Step::Commit => {
                    play::commit_3d(game_addr(play::LISTENER_3D_RVA) as u32)?;
                }
                Step::Frequency(at) => {
                    set::set_buffer_frequency(voice(at))?;
                }
                Step::Volume(at) => {
                    set::set_buffer_volume(voice(at))?;
                }
                Step::Pan(at) => {
                    set::set_buffer_pan(voice(at))?;
                }
                Step::Place(at) => {
                    set::set_buffer_place(voice(at))?;
                }
                Step::Rewind(at) => {
                    play::rewind_buffer(voice(at))?;
                }
                Step::Play(at) => {
                    play::play_buffer_in_game(voice(at))?;
                }
                Step::Stop(at) => {
                    play::stop_buffer(voice(at))?;
                }
            }
            Ok(true)
        });
        store(&l, &room);
        done
    }
}
