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

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::update_voices", rva(0x40AC77), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODE_MIXER: u32 = 0x4_0000;
const A3D_WAY: usize = 0x40CB2A;
const MIXER_WAY: usize = 0x40BC05;
const ENVIRONMENT_RVA: usize = rva(0x6D1DD4); // dword_6D1DD4
const ENVIRONMENT_KEPT_RVA: usize = rva(0x506C80); // dword_506C80

/// Its callees: 0 `is_playing` (the zero flag: not playing), 1 the EAX room, 2
/// `commit_3d`, 3 frequency, 4 volume, 5 pan, 6 place, 7 rewind, 8 play, 9 stop, 10 the
/// A3D way, 11 the mixer's way; esi the voice for 0 and 3 .. 9.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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
pub unsafe fn update_voices(v: &Vars, part: Part) -> Result<(), Refused> {
    use rc2k_core::audio::voices::{Lists, Room, Step, update_buffers};
    unsafe {
        if read(v.a3d) == 1 {
            return part(10, &mut Regs::default());
        }
        let mode = read(v.mode);
        if mode & crate::audio::make_buffer::MODE_OWN_BUFFERS == 0 {
            if mode & MODE_MIXER != 0 {
                part(11, &mut Regs::default())?;
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
            part(which, &mut r)?;
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
    } else if let Err(e) = unsafe { update_voices(&Vars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
    // The original thread's turns (shadow mode); the Rust thread checks in its own turn.
    crate::audio::stray_voices::check();
}

// ---- self-test ----

static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
/// Which voices' buffers still play, by their number in the test's block.
static STILL_PLAYING: Mutex<u32> = Mutex::new(0);
static BASE: Mutex<u32> = Mutex::new(0);

fn handler(place: u32, r: &mut Regs) {
    let esi = if place == 1 || place == 2 { 0 } else { r.esi };
    NOTED.lock().unwrap().push((place, esi));
    if place == 0 {
        let n = (r.esi - *BASE.lock().unwrap()) / std::mem::size_of::<Voice>() as u32;
        r.set_zero(*STILL_PLAYING.lock().unwrap() & (1 << n) == 0);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const N: usize = 10;
    let mut voices = vec![Voice::default(); N];
    let mut channels = vec![0u32; N];
    let mut vars = Box::new([0u32; 10]);
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        a3d: at,
        mode: at + 4,
        playing: at + 8,
        stopping: at + 12,
        used: at + 16,
        commit: at + 20,
        room_asked: at + 24,
        room_set: at + 28,
        environment_kept: at + 32,
        environment: at + 36,
    };
    let calls = [
        0x91, 0x115, 0x12E, 0x13C, 0x1A4, 0x1AB, 0x1B2, 0x1B9, 0x1CA, 0x1D5, 0x21B,
    ];
    let places = [
        (0x40B8E7, 0),
        (0x40D26B, 1),
        (0x40C6A9, 2),
        (0x40B97A, 3),
        (0x40B99D, 4),
        (0x40B9C6, 5),
        (0x40C676, 6),
        (0x40B909, 7),
        (0x40B922, 8),
        (0x40B963, 9),
    ];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40AC77,
            len: 0x242,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &calls,
            jumps_out: &[0x7, 0x23],
        },
        &|callee| {
            places
                .iter()
                .find(|&&(a, _)| rva(a) == callee)
                .map(|&(_, p)| stand_in(p))
        },
        &[
            (0x506D60, v.a3d),
            (0x4F82BC, v.mode),
            (0x4F8340, v.playing),
            (0x4F8344, v.stopping),
            (0x506350, v.used),
            (0x4F82D0, v.commit),
            (0x506C78, v.room_asked),
            (0x506C7C, v.room_set),
            (0x506C80, v.environment_kept),
            (0x6D1DD4, v.environment),
        ],
        22,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let base = voices.as_mut_ptr() as u32;
    *BASE.lock().unwrap() = base;
    let vat = |n: usize| base + (n * std::mem::size_of::<Voice>()) as u32;
    let mut rng = 0xC0FF_EE01u32;
    for case in 0..200 {
        let seed = rng;
        let mut next = || {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            rng >> 8
        };
        // A state: each voice on the playing list, the stopping list or neither.
        let mut setup: Vec<(u32, Voice)> = Vec::new();
        for n in 0..N {
            let list = next() % 3;
            let x = Voice {
                // Only voices on neither list go without a buffer: the Rust version refuses
                // the others.
                buffer: if list == 2 && next() % 3 == 0 {
                    0
                } else {
                    0x7000 + n as u32
                },
                buffer_3d: if next() % 2 == 0 {
                    0
                } else {
                    0x8000 + n as u32
                },
                kind: [0, 1, 3, 0][next() as usize % 4],
                state: next() % 3,
                dirty: next() % 0x40,
                ..Voice::default()
            };
            setup.push((list, x));
        }
        let mode = [0x1_1221u32, 0x11_1220, 0x111_1220, 0, 0x1_0000][next() as usize % 5];
        let still = next() & 0x3FF;
        let room = [next() % 3, next() % 3, next() % 2, next() % 2];
        let commit = next() % 2;
        let prepare = |voices: &mut Vec<Voice>, channels: &mut Vec<u32>, vars: &mut [u32; 10]| {
            let (mut playing, mut stopping) = (Vec::new(), Vec::new());
            for (n, (list, x)) in setup.iter().enumerate() {
                voices[n] = *x;
                channels[n] = 1;
                voices[n].channel = if *list == 0 || n % 4 == 0 {
                    channels.as_ptr() as u32 + n as u32 * 4
                } else {
                    0
                };
                match list {
                    0 => playing.push(n),
                    1 => stopping.push(n),
                    _ => {}
                }
            }
            for l in [&playing, &stopping] {
                for (i, &n) in l.iter().enumerate() {
                    voices[n].next = l.get(i + 1).map_or(0, |&m| vat(m));
                }
            }
            *vars = [
                0,
                mode,
                playing.first().map_or(0, |&n| vat(n)),
                stopping.first().map_or(0, |&n| vat(n)),
                playing.len() as u32,
                commit,
                room[0],
                room[1],
                room[2],
                room[3],
            ];
            *STILL_PLAYING.lock().unwrap() = still;
            NOTED.lock().unwrap().clear();
        };
        prepare(&mut voices, &mut channels, &mut vars);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            voices.clone(),
            channels.clone(),
            *vars,
            std::mem::take(&mut *NOTED.lock().unwrap()),
        );
        prepare(&mut voices, &mut channels, &mut vars);
        let done = unsafe {
            update_voices(&v, &mut |p, r| {
                handler(p, r);
                Ok(())
            })
        };
        let mine = (
            voices.clone(),
            channels.clone(),
            *vars,
            std::mem::take(&mut *NOTED.lock().unwrap()),
        );
        t.check(
            done.is_ok() && theirs == mine && ro.preserved_except(&Regs::SENTINEL, 0),
            || {
                format!(
                    "case {case} (seed {seed:#x}), mode {mode:#x}: rust {done:?}; original {:x?} calls {:x?}; rust {:x?} calls {:x?}",
                    theirs.2, theirs.3, mine.2, mine.3
                )
            },
        );
    }
    t.finish()
}
