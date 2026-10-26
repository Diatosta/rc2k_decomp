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

use crate::audio::buffer_play::Leaves;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 9] = [0x28, 0x51, 0x60, 0x7C, 0x88, 0x94, 0xA3, 0xB8, 0xD0];

pub static HOOK: RawHook = RawHook::new("audio::take_commands", rva(0x40A8A4), thunk, &ORIGINAL)
    .private_copy(0x12D)
    .relocate_calls(&CALLS)
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const COUNT_RVA: usize = rva(0x506A1C); // g_soundCommandCount
pub const READ_RVA: usize = rva(0x506A20); // g_soundCommandRead
pub const LIST_LOADED_RVA: usize = rva(0x50695C); // g_soundListLoaded
const SET_MODE: usize = 0x40A426; // audio::set_mode
/// The callees in the order of `CALLS`: their places for `Part`.
const CALLEES: [usize; 9] = [
    0x40A9D1, 0x40AEB9, 0x40AF4B, SET_MODE, 0x40AFD6, 0x40AFD9, 0x40B016, 0x40C194, 0x40C4DD,
];

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

/// A command's call, by its place in `CALLEES`, with the registers the original has there.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Converted: `rc2k_core::audio::commands::decode` and `next_place`; the registers carry
/// over from one command to the next as the original's do.
pub fn take_commands(entry: &Regs, v: &Vars, part: Part) -> Result<(), Refused> {
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
                    part(0, &mut r)?;
                }
                Command::Room(room) => {
                    r.eax = room;
                    write(v.eax_room, room);
                }
                Command::Load(list) => {
                    r.eax = list;
                    write(v.list, list);
                    part(1, &mut r)?;
                }
                Command::Free => {
                    part(2, &mut r)?;
                    write(v.list, 0);
                }
                Command::Mode(mode) => {
                    r.eax = mode;
                    part(3, &mut r)?;
                }
                Command::Six => part(4, &mut r)?,
                Command::StopAll => part(5, &mut r)?,
                Command::Volume(volume) => {
                    r.eax = volume;
                    part(6, &mut r)?;
                }
                Command::Stream {
                    path,
                    volume,
                    action,
                } => {
                    (r.eax, r.ecx, r.edx) = (path, volume, action);
                    part(7, &mut r)?;
                }
                Command::Music {
                    tune,
                    next,
                    volume,
                    action,
                } => {
                    (r.eax, r.ebx, r.ecx, r.edx) = (tune, next, volume, action);
                    part(8, &mut r)?;
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
                &mut crate::audio::free_voices::game_part,
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

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    take_commands(r, &Vars::game(), &mut game_part)?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    // The Rust version in shadow mode too: the original's `dec` of the count is not atomic,
    // and the game thread's `inc` races it on a machine of more than one core (a command
    // lost or an old one taken again: the stuck sounds, 2026-10-21; notes/bugs.md).
    HOOK.count_call();
    if let Err(e) = unsafe { rust(r) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// Each call: the place and every register.
static NOTED: Mutex<Vec<[u32; 8]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Used through their addresses only (the code under test writes them).
    let mut records = vec![[0u32; 8]; 0x400];
    let mut vars = Box::new([0u32; 4]);
    let rec = records.as_mut_ptr() as u32;
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        count: at,
        read: at + 4,
        records: rec,
        eax_room: at + 8,
        list: at + 12,
    };
    let entry = {
        let mut e = [0x60u8, 0x83, 0x3D, 0, 0, 0, 0, 0x00];
        e[3..7].copy_from_slice(&(game_addr(COUNT_RVA) as u32).to_le_bytes());
        e
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A8A4,
            len: 0x12D,
            entry: &entry,
            calls: &CALLS,
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x506A1C, v.count),
            (0x506A20, v.read),
            (0x9566F8, v.records),
            (0x506C78, v.eax_room),
            (0x50695C, v.list),
        ],
        14,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    // A queue's records: the number, eax, ebx, ecx, edx, esi, edi, ebp.
    let queue: Vec<[u32; 8]> = [1u32, 2, 3, 9, 5, 4, 6, 7, 8, 0xB, 5, 0, 0xC, 1, 0xA, 9, 1]
        .iter()
        .enumerate()
        .map(|(n, &c)| {
            let n = n as u32;
            [
                c,
                0xFFFF_8000 + n * 0x1111,
                0xB0 + n,
                0xC0 + n,
                0xD0 + n,
                0xE0 + n,
                0xF0,
                0xA0,
            ]
        })
        .collect();
    for start in [0u32, 0x3FA] {
        for count in [0u32, 1, 4, 10, 14, 15, 16, 17, 20, 0x30] {
            for edx in [1u32, 0] {
                let run = |rust: bool| {
                    unsafe {
                        let r = rec as *mut [u32; 8];
                        for n in 0..0x400 {
                            r.add(n).write_volatile([0; 8]);
                        }
                        for (n, q) in queue.iter().enumerate() {
                            r.add((start as usize + n) % 0x400).write_volatile(*q);
                        }
                        (at as *mut [u32; 4]).write_volatile([count, start, 0x77, 0x66]);
                    }
                    NOTED.lock().unwrap().clear();
                    let before = Regs {
                        edx,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let done = if rust {
                        take_commands(&before, &v, &mut |p, r| {
                            handler(p, r);
                            Ok(())
                        })
                        .is_ok()
                    } else {
                        unsafe { call_game(&mut ro, copy.addr()) };
                        ro.preserved_except(&before, 0)
                    };
                    (done, std::mem::take(&mut *NOTED.lock().unwrap()), unsafe {
                        (at as *const [u32; 4]).read_volatile()
                    })
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0 && theirs == mine, || {
                    format!(
                        "read from {start:#x}, count {count}, edx {edx}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    drop((records, vars));
    t.finish()
}
