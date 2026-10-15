//! `audio::load_sounds` (0x40AEB9): command 3: a list of sounds loaded.
//!
//! In:  eax = the list (the sender's eax): sound numbers, -2 passed over, ended by -1
//! Out: every register preserved
//!
//! For each sound whose record has a file (`data_key`) and `copies` above 0 (signed), that
//! many voices while fewer than 0x200 are in use: `audio::find_shared_voice`; none:
//! `audio::read_sample_of` (a carry ends the sound's voices); `audio::new_voice` (the
//! record's first voice if it has none); `audio::fill_voice` with the shared voice;
//! `audio::sample_mode` on `g_soundMode`; `audio::prepare_sample`; `audio::keep_sample`;
//! `audio::make_buffer` and `audio::fill_buffer` (their carries not looked at); the
//! voice's flags get `LOADED`. After each entry of the list, `g_quitting` 1 ends it. The
//! callees get the registers the original has at each call (ebx the record, esi the voice,
//! eax the shared voice or the mode): `make_buffer` hands ebx on to the EAX set-up. One
//! caller, `audio::take_commands`.
//! Refuses, before anything is done: a list without its -1 within 0x1000 entries, or with
//! a sound of 0x1CD or more; A3D.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variables are the
//! test's and whose nine calls follow a script and note their registers, 40 cases.

use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::load_sounds", rva(0x40AEB9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const SKIP: u32 = 0xFFFF_FFFE;
const END: u32 = 0xFFFF_FFFF;
const LIST_ROOM: u32 = 0x1000;

/// Its variables: the records, the voices in use, the mode, the quit switch.
#[derive(Clone, Copy)]
pub struct Vars {
    pub samples: u32,
    pub voice_count: u32,
    pub mode: u32,
    pub quitting: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            samples: a(SAMPLES_RVA),
            voice_count: a(VOICE_COUNT_RVA),
            mode: a(SOUND_MODE_RVA),
            quitting: a(QUITTING_RVA),
        }
    }
}

/// Its nine callees, in the order of the doc above (0 `find_shared_voice` .. 8
/// `fill_buffer`), on the registers as the original has them; true: the carry they leave.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

pub unsafe fn load_sounds(list: u32, v: &Vars, part: Part) -> Result<(), Refused> {
    if list == 0 {
        return Ok(());
    }
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D").into());
    }
    unsafe {
        // The whole list first.
        let mut n = 0;
        loop {
            let entry = read(list + n * 4);
            if entry == END {
                break;
            }
            if entry != SKIP && entry >= SAMPLES {
                return Err(BadInput("a sound of 0x1CD or more in the list").into());
            }
            n += 1;
            if n == LIST_ROOM {
                return Err(BadInput("a list without its end").into());
            }
        }
        let mut at = list;
        loop {
            let entry = read(at);
            at += 4;
            if entry == END {
                return Ok(());
            }
            let record_at = v.samples + entry.wrapping_mul(0x40);
            if entry != SKIP {
                let record = &mut *(record_at as *mut SampleRecord);
                if record.data_key != 0 && record.copies > 0 {
                    let mut count = record.copies as u32;
                    let base = Regs {
                        ebx: record_at,
                        edx: at,
                        ..Regs::default()
                    };
                    loop {
                        if read(v.voice_count) == VOICES {
                            break;
                        }
                        let mut r = Regs { ecx: count, ..base };
                        part(0, &mut r)?;
                        let shared = r.eax;
                        if shared == 0 {
                            let mut r = Regs { ecx: count, ..base };
                            if part(1, &mut r)? {
                                break;
                            }
                        }
                        let mut r = Regs {
                            eax: shared,
                            ecx: count,
                            ..base
                        };
                        part(2, &mut r)?;
                        let voice_at = r.esi;
                        if record.first_voice == 0 {
                            record.first_voice = voice_at;
                        }
                        let with = |eax| Regs {
                            eax,
                            ecx: count,
                            esi: voice_at,
                            ..base
                        };
                        part(3, &mut with(shared))?;
                        let mut r = with(read(v.mode));
                        part(4, &mut r)?;
                        let mode = r.eax;
                        for p in 5..=8 {
                            part(p, &mut with(mode))?;
                        }
                        voice(voice_at).flags |= LOADED;
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                    }
                }
            }
            if read(v.quitting) == 1 {
                return Ok(());
            }
        }
    }
}

/// The game's callees: the Rust versions.
pub fn game_part(which: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::*;
    unsafe {
        let record = r.ebx;
        match which {
            0 => {
                let count: u32 = crate::framework::game_read(VOICE_COUNT_RVA);
                r.eax = samples::find_shared_voice(
                    &*(record as *const SampleRecord),
                    game_addr(VOICES_RVA) as u32,
                    count.min(VOICES),
                );
            }
            1 => {
                let found =
                    read_sample_of::read_sample_of(&*(record as *const SampleRecord), &mut |p| {
                        samples::read_sample_in_game(p as *const u8)
                    })?;
                return Ok(!found);
            }
            2 => r.esi = new_voice::new_voice_in_game()?,
            3 => fill_voice::fill_voice_in_game(r.esi, record, r.eax)?,
            4 => r.eax = sample_mode::sample_mode_of(r.eax, voice(r.esi).sound)?,
            5 => samples::prepare_sample(voice(r.esi), r.eax)?,
            6 => keep_sample::keep_sample(voice(r.esi), &mut |n| keep_sample::allocate_in_game(n))?,
            7 => {
                let vars = make_buffer::Vars::game();
                let made = make_buffer::make_buffer(r.esi, r.ebx, &vars, &mut |p, r| {
                    make_buffer::game_part(p, r)
                })?;
                return Ok(!made);
            }
            _ => {
                let parts = game_addr(fill_buffer::LOCKED_PARTS_RVA) as u32;
                let lost = game_addr(BUFFER_LOST_RVA) as u32;
                return Ok(!fill_buffer::fill_buffer(voice(r.esi), parts, lost)?);
            }
        }
    }
    Ok(false)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { load_sounds(r.eax, &Vars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// What the stand-ins did: (callee, eax, ebx, ecx, edx, esi).
static NOTED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());
/// The script: the voice count's place, the voices' base, which records share, which
/// reads fail.
static SCRIPT: Mutex<Script> = Mutex::new(Script {
    count: 0,
    voices: 0,
    shared_every: 0,
    fail_every: 0,
    calls: 0,
});

struct Script {
    count: u32,
    voices: u32,
    shared_every: u32,
    fail_every: u32,
    calls: u32,
}

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi]);
    let mut s = SCRIPT.lock().unwrap();
    s.calls += 1;
    let calls = s.calls;
    unsafe {
        match place {
            0 => {
                r.eax = if s.shared_every != 0 && calls.is_multiple_of(s.shared_every) {
                    0x5000_0000 | calls
                } else {
                    0
                }
            }
            1 => r.set_carry(s.fail_every != 0 && calls.is_multiple_of(s.fail_every)),
            2 => {
                let n = read(s.count);
                write(s.count, n + 1);
                r.esi = s.voices + (n % 8) * std::mem::size_of::<Voice>() as u32;
            }
            4 => r.eax = r.eax.rotate_left(4) ^ calls,
            _ => r.set_carry(calls.is_multiple_of(3)),
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if crate::audio::buffer_settings::a3d() {
        return t.skip("A3D is on");
    }
    // The records, eight voices, the four variables.
    let mut records = vec![SampleRecord::default(); SAMPLES as usize];
    let mut voices = vec![Voice::default(); 8];
    let mut vars = Box::new([0u32; 4]);
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        samples: records.as_mut_ptr() as u32,
        voice_count: at,
        mode: at + 4,
        quitting: at + 8,
    };
    let calls = [0x3C, 0x45, 0x4C, 0x5A, 0x64, 0x69, 0x6E, 0x73, 0x78];
    let targets = [
        0x40B027, 0x40B1BE, 0x40B04C, 0x40B06E, 0x40B398, 0x40B3F5, 0x40B192, 0x40B5CF, 0x40B82B,
    ];
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40AEB9,
            len: 0x92,
            entry: &[0x60, 0x85, 0xC0, 0x0F, 0x84, 0x87, 0x00, 0x00, 0x00],
            calls: &calls,
            jumps_out: &[],
        },
        &|callee| {
            targets
                .iter()
                .position(|&t| rva(t) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x50A7A4, v.samples),
            (0x4F8348, v.voice_count),
            (0x4F82BC, v.mode),
            (0x51837C, v.quitting),
        ],
        4,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let lists: [&[u32]; 5] = [
        &[1, 2, 3, END],
        &[SKIP, 5, SKIP, 0x1CC, END],
        &[END],
        &[7, 7, 0, END],
        &[4, 6, 8, 9, 10, END],
    ];
    let mut case = 0;
    for list in lists {
        for (shared_every, fail_every) in [(0u32, 0u32), (2, 0), (0, 3), (5, 4)] {
            for start_count in [0u32, 0x1FE] {
                case += 1;
                let list = list.to_vec();
                let run = |rust: bool,
                           records: &mut Vec<SampleRecord>,
                           voices: &mut Vec<Voice>,
                           vars: &mut [u32; 4]| {
                    for (n, r) in records.iter_mut().enumerate() {
                        *r = SampleRecord {
                            number: n as u32,
                            data_key: if n % 5 == 3 { 0 } else { 0x9000 + n as u32 },
                            copies: [1, 2, 3, 0, -1, 2][n % 6],
                            first_voice: if n % 4 == 1 { 0x7777 } else { 0 },
                            ..SampleRecord::default()
                        };
                    }
                    voices.iter_mut().for_each(|x| *x = Voice::default());
                    *vars = [start_count, 0x2441220, if case % 7 == 3 { 1 } else { 0 }, 0];
                    *SCRIPT.lock().unwrap() = Script {
                        count: v.voice_count,
                        voices: voices.as_mut_ptr() as u32,
                        shared_every,
                        fail_every,
                        calls: 0,
                    };
                    NOTED.lock().unwrap().clear();
                    let before = Regs {
                        eax: list.as_ptr() as u32,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let done = if rust {
                        unsafe {
                            load_sounds(before.eax, &v, &mut |p, r| {
                                handler(p, r);
                                Ok(r.carry())
                            })
                        }
                        .map(|_| true)
                    } else {
                        unsafe { call_game(&mut ro, copy.addr()) };
                        Ok(ro.preserved_except(&before, 0))
                    };
                    let noted = std::mem::take(&mut *NOTED.lock().unwrap());
                    (
                        done,
                        records.clone(),
                        voices.clone(),
                        // Volatile: the code under test wrote it behind the reference.
                        unsafe { std::ptr::from_mut(vars).read_volatile() },
                        noted,
                    )
                };
                let theirs = run(false, &mut records, &mut voices, &mut vars);
                let mine = run(true, &mut records, &mut voices, &mut vars);
                // Each callee's registers as far as it reads them (eax, ebx, esi; ecx and edx
                // are the original's loop counter and list cursor, which none reads).
                let strip = |n: &Vec<[u32; 6]>| -> Vec<[u32; 4]> {
                    n.iter()
                        .map(|c| {
                            let (eax, ebx, esi) = match c[0] {
                                0 | 1 => (false, true, false),
                                2 => (false, false, false),
                                3 => (true, true, true),
                                4 | 5 => (true, false, true),
                                7 => (false, true, true),
                                _ => (false, false, true),
                            };
                            let keep = |on: bool, x: u32| if on { x } else { 0 };
                            [c[0], keep(eax, c[1]), keep(ebx, c[2]), keep(esi, c[5])]
                        })
                        .collect()
                };
                let ok = matches!(theirs.0, Ok(true))
                    && mine.0.is_ok()
                    && theirs.1 == mine.1
                    && theirs.2 == mine.2
                    && theirs.3 == mine.3
                    && strip(&theirs.4) == strip(&mine.4);
                t.check(ok, || {
                    format!(
                        "case {case}: list {list:x?}, shared every {shared_every}, failing every {fail_every}, count {start_count:#x}: original {:?} {:x?} calls {:x?}; rust {:?} {:x?} calls {:x?}",
                        theirs.0, theirs.3, strip(&theirs.4), mine.0, mine.3, strip(&mine.4)
                    )
                });
            }
        }
    }
    t.finish()
}
