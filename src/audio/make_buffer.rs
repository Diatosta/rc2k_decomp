//! `audio::make_buffer` (0x40B5CF): a voice's DirectSound buffer made.
//!
//! In:  esi = the voice
//! Out: carry = not made; every register preserved
//!
//! Nothing (carry clear) unless voices have buffers of their own (`g_soundMode` bit
//! 0x10000). No `g_directSound`: carry. A voice that shares data gets a duplicate of the
//! other's buffer (`DuplicateSoundBuffer`, +0x14); if that fails, or for any other voice,
//! `audio::describe_wave_format` and `CreateSoundBuffer` (+0x0C) with
//! `g_bufferDescription`: size 0x14, flags 0xE0 (frequency, pan and volume controls), + 8
//! (in software) unless 3D with bit 0x800000, + 2 (static) for the voice's flags bit 0, +
//! 0x10 (3D) in 3D, the length, `g_waveFormat`; failing: "AUD: 1-DSBCreate failed" and the
//! result to the game's log, carry. Then the volume, pan and frequency
//! (`audio::set_buffer_*`). In 3D: the 3D interface (`QueryInterface` with
//! `g_iidSound3dBuffer`; failing: "AUD: 1-DSBQuery3D failed" logged, carry), the place,
//! `audio::set_buffer_min_distance`, `g_commit3d` 1, `audio::open_voice_manager` with bit
//! 0x800000, with EAX (bit 0x1000000) `audio::open_eax` and, unless that sets the carry,
//! `audio::set_eax_room` (the openings read through ebx, which is the caller's, the
//! frequency after the create path, or the 3D buffer's place after the voice manager's
//! call, as the original leaves it; `open_eax` refuses one that is no pointer). Callers:
//! command 3 and `audio::start`.
//! Refuses A3D (the chunk at 0x40CC88), and what the settings refuse, checked first.
//! Shadow mode runs the original alone. Self-test: a redirected copy on a DirectSound of
//! the test's (`audio/test_sound.rs`) whose 3D calls and logging note their registers, 48
//! cases.

use crate::audio::buffer_settings::{a3d, set_buffer_frequency, set_buffer_pan, set_buffer_volume};
use crate::audio::describe_wave_format::{WAVE_FORMAT_RVA, describe_wave_format};
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::make_buffer", rva(0x40B5CF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const DIRECT_SOUND_RVA: usize = rva(0x506A88); // g_directSound
pub const DESCRIPTION_RVA: usize = rva(0x506B1C); // g_bufferDescription
const IID_3D_RVA: usize = rva(0x506A58); // g_iidSound3dBuffer
const CREATE_FAILED_RVA: usize = rva(0x4DA6E4); // "AUD: 1-DSBCreate failed"
const QUERY_FAILED_RVA: usize = rva(0x4DA6FC); // "AUD: 1-DSBQuery3D failed"
pub const MODE_OWN_BUFFERS: u32 = 0x1_0000;
const MODE_HARDWARE_3D: u32 = 0x80_0000;
const MODE_EAX: u32 = 0x100_0000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BufferDescription {
    pub size: u32,
    pub flags: u32,
    pub bytes: u32,
    pub reserved: u32,
    pub format: u32,
}

/// Where its variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub mode: u32,
    pub direct_sound: u32,
    pub description: u32,
    pub commit: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            mode: game_addr(SOUND_MODE_RVA) as u32,
            direct_sound: game_addr(DIRECT_SOUND_RVA) as u32,
            description: game_addr(DESCRIPTION_RVA) as u32,
            commit: game_addr(COMMIT_RVA) as u32,
        }
    }
}

/// The parts it calls by the game's addresses: 0 the place, 1 the minimum distance,
/// 2 `sub_40C6B5`, 3 `sub_40D27F` (true: carry), 4 `sub_40D26B`; 5 a text to the log,
/// 6 a number to the log with a line's end.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// True for the carry clear. `ebx`: the caller's, which the original leaves in ebx (or
/// replaces, as below) when it calls the EAX parts: they read through it.
pub unsafe fn make_buffer(at: u32, ebx: u32, vars: &Vars, part: Part) -> Result<bool, Refused> {
    unsafe {
        if a3d() {
            return Err(BadInput("A3D (the chunk at 0x40CC88)").into());
        }
        let mode = read(vars.mode);
        if mode & MODE_OWN_BUFFERS == 0 {
            return Ok(true);
        }
        let ds = read(vars.direct_sound);
        if ds == 0 {
            return Ok(false);
        }
        let v = voice(at);
        // What the settings would refuse, before anything is made.
        let level = (v.volume as i32).wrapping_mul(v.volume_factor as i32) >> 6;
        if !(0..=0x40).contains(&level) || !(-0x40..=0x40).contains(&v.pan) {
            return Err(BadInput("a volume or pan past the volume table").into());
        }
        // ebx as the original has it at each call: the caller's, the frequency after the
        // create path (`mov ebx, [esi+60h]`), the 3D buffer's place after `sub_40C6B5`'s.
        let mut regs = Regs {
            esi: at,
            ebx,
            ..Regs::default()
        };
        let mut made = false;
        if v.source != 0 {
            let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, 0x14));
            if f(ds, voice(v.source).buffer, &mut v.buffer as *mut u32 as u32) == 0 {
                made = true;
            } else {
                v.source = 0;
            }
        }
        if !made {
            regs.ebx = v.frequency;
            describe_wave_format(v.format, v.frequency)?;
            let mut flags = 0xE0;
            if !(mode & MODE_3D != 0 && mode & MODE_HARDWARE_3D != 0) {
                flags |= 8;
            }
            if v.flags & 1 != 0 {
                flags |= 2;
            }
            if mode & MODE_3D != 0 {
                flags |= 0x10;
            }
            (vars.description as *mut BufferDescription).write_unaligned(BufferDescription {
                size: 0x14,
                flags,
                bytes: v.length,
                reserved: 0,
                format: game_addr(WAVE_FORMAT_RVA) as u32,
            });
            let f: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, 0x0C));
            let r = f(ds, vars.description, &mut v.buffer as *mut u32 as u32, 0);
            if r != 0 {
                return log_failure(part, CREATE_FAILED_RVA, r);
            }
        }
        set_buffer_volume(v)?;
        set_buffer_pan(v)?;
        set_buffer_frequency(v)?;
        if mode & MODE_3D != 0 {
            let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(v.buffer, 0));
            let r = f(
                v.buffer,
                game_addr(IID_3D_RVA) as u32,
                &mut v.buffer_3d as *mut u32 as u32,
            );
            if r != 0 {
                return log_failure(part, QUERY_FAILED_RVA, r);
            }
            part(0, &mut regs)?;
            part(1, &mut regs)?;
            write(vars.commit, 1);
            if mode & MODE_HARDWARE_3D != 0 {
                regs.ebx = &mut v.buffer_3d as *mut u32 as u32;
                let mut r = regs;
                part(2, &mut r)?;
            }
            if mode & MODE_EAX != 0 {
                let mut r = regs;
                if !part(3, &mut r)? {
                    let mut r = regs;
                    part(4, &mut r)?;
                }
            }
        }
        Ok(true)
    }
}

fn log_failure(part: Part, text: usize, result: i32) -> Result<bool, Refused> {
    let mut r = Regs {
        eax: game_addr(text) as u32,
        ..Regs::default()
    };
    part(5, &mut r)?;
    let mut r = Regs {
        eax: result as u32,
        ..Regs::default()
    };
    part(6, &mut r)?;
    Ok(false)
}

/// The parts' addresses, for the self-test's stand-ins: `set_buffer_place`,
/// `set_buffer_min_distance`, `open_voice_manager`, `open_eax`, `set_eax_room`, the two logs.
const PARTS: [usize; 7] = [
    0x40C676, 0x40C69A, 0x40C6B5, 0x40D27F, 0x40D26B, 0x401427, 0x401626,
];

/// The game's parts: the Rust versions (the carry of 3 returned).
pub fn game_part(which: u32, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        match which {
            0 => {
                crate::audio::buffer_settings::set_buffer_place(voice(r.esi))?;
            }
            1 => {
                let v = &*(r.esi as *const Voice);
                r.eax = crate::audio::set_buffer_min_distance::set_buffer_min_distance(v)? as u32;
            }
            2 => {
                crate::audio::open_voice_manager::open_voice_manager(r.ebx)?;
            }
            3 => return Ok(!crate::audio::open_eax::open_eax(r.ebx)?),
            4 => r.eax = crate::audio::set_eax_room::in_game()?,
            5 => crate::logfile::log_write::log_write(r.eax as *const u8, false, 1),
            _ => crate::logfile::log_hex8::log_hex8(r.eax, true, 1),
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
    match unsafe { make_buffer(r.esi, r.ebx, &Vars::game(), &mut game_part) } {
        Ok(made) => r.set_carry(!made),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<(u32, u32, u32)>> = Mutex::new(Vec::new());
static EAX_CARRY: AtomicBool = AtomicBool::new(false);

fn handler(place: u32, r: &mut Regs) {
    // The registers its parts read: esi and ebx (the EAX parts read through ebx), eax for
    // the log's.
    let what = if place >= 5 {
        (r.eax, 0)
    } else {
        (r.esi, r.ebx)
    };
    NOTED.lock().unwrap().push((place, what.0, what.1));
    if place == 3 {
        r.set_carry(EAX_CARRY.load(Ordering::SeqCst));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if a3d() {
        return t.skip("A3D is on");
    }
    let mut vars = Box::new([0u32; 1 + 1 + 5 + 1]);
    let at = vars.as_mut_ptr() as u32;
    let v_vars = Vars {
        mode: at,
        direct_sound: at + 4,
        description: at + 8,
        commit: at + 28,
    };
    let mut a3d_var = Box::new(0u32);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B5CF,
            len: 0x198,
            entry: &[0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
            calls: &[
                0x5B, 0x6D, 0xF5, 0xFA, 0xFF, 0x125, 0x12A, 0x148, 0x159, 0x160, 0x170, 0x177,
                0x186, 0x18D,
            ],
            jumps_out: &[0x7],
        },
        &|callee| {
            PARTS
                .iter()
                .position(|&p| rva(p) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x506D60, &mut *a3d_var as *mut u32 as u32),
            (0x4F82BC, v_vars.mode),
            (0x506A88, v_vars.direct_sound),
            (0x506B1C, v_vars.description),
            (0x506B20, v_vars.description + 4),
            (0x506B24, v_vars.description + 8),
            (0x506B2C, v_vars.description + 16),
            (0x4F82D0, v_vars.commit),
        ],
        21,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = crate::audio::test_sound::TestSound::new();
    let wave = [(game_addr(WAVE_FORMAT_RVA) as u32, 0x12usize)];
    let saved_wave = crate::service::harness::capture(&wave);
    let mut case = 0;
    for mode in [0u32, 0x1_1221, 0x11_1220, 0x91_1220, 0x191_1220, 0x111_1220] {
        for (create, duplicate, query) in [(0, 0, 0), (-1, 0, 0), (0, -1, 0), (0, 0, -1)] {
            for (shared, eax_carry) in [(false, false), (true, true)] {
                case += 1;
                let mut source = Voice {
                    buffer: sound.buffer(0),
                    ..Voice::default()
                };
                let make = |source: &mut Voice| Voice {
                    flags: if case % 2 == 0 { 1 } else { 2 },
                    format: 0x221,
                    length: 0x1234,
                    volume: 0x30,
                    volume_factor: 0x40,
                    pan: -12,
                    frequency: 22050,
                    source: if shared {
                        source as *mut Voice as u32
                    } else {
                        0
                    },
                    place: [1, 2, 3],
                    min_distance: 0x3F80_0000,
                    ..Voice::default()
                };
                let results = crate::audio::test_sound::Results {
                    create,
                    duplicate,
                    query,
                    ..Default::default()
                };
                EAX_CARRY.store(eax_carry, Ordering::SeqCst);
                let mut run = |rust: bool, source: &mut Voice| {
                    let mut v = make(source);
                    *vars = [mode, sound.sound(), 0, 0, 0, 0, 0, 0];
                    NOTED.lock().unwrap().clear();
                    sound.begin(results);
                    let before = Regs {
                        esi: &mut v as *mut Voice as u32,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let made = if rust {
                        unsafe {
                            make_buffer(before.esi, before.ebx, &v_vars, &mut |p, r| {
                                handler(p, r);
                                Ok(r.carry())
                            })
                        }
                        .map(|m| (m, true))
                    } else {
                        unsafe { call_game(&mut ro, copy.addr()) };
                        Ok((!ro.carry(), ro.preserved_except(&before, 0)))
                    };
                    (
                        made,
                        v,
                        *vars,
                        sound.noted(),
                        std::mem::take(&mut *NOTED.lock().unwrap()),
                    )
                };
                let theirs = run(false, &mut source);
                let mine = run(true, &mut source);
                let ok = matches!(theirs.0, Ok((_, true)))
                    && mine.0.as_ref().ok().map(|m| m.0) == theirs.0.as_ref().ok().map(|m| m.0)
                    && theirs.1 == mine.1
                    && theirs.2 == mine.2
                    && theirs.3 == mine.3
                    && theirs.4 == mine.4;
                t.check(ok, || {
                    format!(
                        "case {case}: mode {mode:#x}, results {create} {duplicate} {query}, shared {shared}: original {:?} {:x?} {:x?} {:x?}; rust {:?} {:x?} {:x?} {:x?}",
                        theirs.0, theirs.2, theirs.3, theirs.4, mine.0, mine.2, mine.3, mine.4
                    )
                });
            }
        }
    }
    crate::service::harness::restore(&wave, &saved_wave);
    t.finish()
}
