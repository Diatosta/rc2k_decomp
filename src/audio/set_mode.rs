//! `audio::set_mode` (0x40A426): command 5, the sound mode changed (menus <-> a race).
//!
//! In:  eax = the new mode; ebx, ecx = handed to `audio::stop_streams`, with action 0 (the
//!      original hands on edx: section 16, fixed 2026-10-20)
//! Out: carry = failed ("SAM: FAIL"); every register preserved
//!
//! "SAM:" and the mode logged; `audio::stop_streams`, `audio::free_voices`,
//! `audio::close_way`, `audio::release_primary`; `g_soundMode` the new mode; the channels:
//! 8 (0x10 with bit 0x400000), held to the free hardware buffers of the mode's kind (2D or
//! 3D: bit 0x10000 all, 0x20000 the other count of `g_soundCaps`) when there are any, and
//! at least 6, into `g_channelCount`; `audio::reopen_sound`, `audio::make_primary` (either
//! failing: `g_openedSoundMode` 0, "SAM: FAIL", the carry), `audio::open_way`, a second's
//! `system::sleep`, `audio::play_primary`; `g_openedSoundMode` the mode, "SAM: OK". Either
//! way `system::restore_window` last (its flags kept around it). Callers: the audio thread's
//! start (the menus' mode), `audio::take_commands` (command 5).
//! Shadow mode runs the original alone. Self-test: a redirected copy whose calls go to
//! stand-ins noting what each reads (no wait, no window), 45 cases; the original's
//! `stop_streams` action taken as 0 (the fix).

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_mode", rva(0x40A426), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place; `nullsub_44` (0x40AC76) is left as it is.
const CALLEES: [usize; 13] = [
    0x401432, // 0 log_write_l2 (eax)
    0x401634, // 1 log_hex8_l2_nl (eax)
    0x40C5E8, // 2 stop_streams (eax .. edx)
    0x40AF4B, // 3 free_voices
    0x40AC58, // 4 close_way
    0x40A61F, // 5 release_primary
    0x40A52A, // 6 reopen_sound (carry)
    0x40A54B, // 7 make_primary (carry)
    0x40AC3A, // 8 open_way
    0x41100C, // 9 system::sleep (eax)
    0x40A775, // 10 play_primary
    0x401439, // 11 log_write_l2_nl (eax)
    0x405A33, // 12 system::restore_window
];
/// Every call of the copy, `nullsub_44`'s (0xC3) too: it runs, so it must be relocated.
const CALLS: [usize; 15] = [
    0x08, 0x0F, 0x16, 0x1B, 0x20, 0x25, 0xA1, 0xA8, 0xAF, 0xB9, 0xBE, 0xC3, 0xD9, 0xE2, 0xFB,
];
const SAM_TEXT: usize = rva(0x4DA6CD); // "SAM:"
const OK_TEXT: usize = rva(0x4DA6D2); // "SAM: OK"
const FAIL_TEXT: usize = rva(0x4DA6DA); // "SAM: FAIL"

#[derive(Clone, Copy)]
pub struct ModeVars {
    pub mode: u32,
    /// `g_soundCaps` (free hardware buffers: +0x24 all, +0x28 static, +0x3C 3D all,
    /// +0x40 3D static).
    pub caps: u32,
    pub channel_count: u32,
    pub opened: u32,
}

impl ModeVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            mode: a(crate::audio::voices::SOUND_MODE_RVA),
            caps: a(crate::audio::sound_device::CAPS_RVA),
            channel_count: a(crate::audio::voices::CHANNEL_COUNT_RVA),
            opened: a(crate::audio::sound_device::OPENED_MODE_RVA),
        }
    }
}

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// True for no carry.
pub fn set_mode(entry: &Regs, v: &ModeVars, part: Part) -> Result<bool, Refused> {
    unsafe {
        let mode = entry.eax;
        let mut c = *entry;
        let mut call = |place: u32, c: &Regs| -> Result<bool, Refused> {
            let mut r = *c;
            part(place, &mut r)
        };
        call(
            0,
            &Regs {
                eax: game_addr(SAM_TEXT) as u32,
                ..c
            },
        )?;
        call(1, &c)?;
        // Fixed (section 16, 2026-10-20): the streams stopped with action 0; the original
        // hands on whatever edx holds (normally the thread's 1, a feed), so the music
        // outlived the DirectSound it played on.
        call(2, &Regs { edx: 0, ..c })?;
        for place in 3..6 {
            call(place, &c)?;
        }
        c.edx = mode;
        write(v.mode, mode);
        let mut channels: i32 = if mode & 0x40_0000 != 0 { 0x10 } else { 8 };
        let (all, other) = if mode & 0x10_0000 == 0 {
            (0x24, 0x28)
        } else {
            (0x3C, 0x40)
        };
        c.ebx = read(v.caps + all);
        let mut held = mode & 0x1_0000 != 0;
        if !held {
            c.ebx = read(v.caps + other);
            held = mode & 0x2_0000 != 0;
        }
        let most = c.ebx as i32;
        if held && most != 0 {
            if channels > most {
                channels = most;
            }
            if channels < 6 {
                channels = 6;
            }
        }
        c.eax = channels as u32;
        write(v.channel_count, c.eax);
        let ok = !call(6, &c)? && !call(7, &c)?;
        if ok {
            call(8, &c)?;
            call(9, &Regs { eax: 1000, ..c })?;
            call(10, &c)?;
            write(v.opened, read(v.mode));
            call(
                11,
                &Regs {
                    eax: game_addr(OK_TEXT) as u32,
                    ..c
                },
            )?;
        } else {
            write(v.opened, 0);
            call(
                11,
                &Regs {
                    eax: game_addr(FAIL_TEXT) as u32,
                    ..c
                },
            )?;
        }
        call(12, &c)?;
        Ok(ok)
    }
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    use crate::audio::voices::*;
    let mode = || unsafe { crate::framework::game_read::<u32>(SOUND_MODE_RVA) };
    unsafe {
        Ok(match place {
            0 => sd::game_part(0, r.eax)?,
            1 => sd::game_part(3, r.eax)?,
            11 => sd::game_part(1, r.eax)?,
            2 => {
                crate::audio::stop_streams::stop_streams(
                    r,
                    &mut crate::audio::stop_streams::game_command,
                )?;
                false
            }
            3 => {
                crate::audio::free_voices::free_voices(
                    &World::game(),
                    &crate::audio::free_voices::game_vars(),
                    &mut crate::audio::free_voices::game_part,
                )?;
                false
            }
            4 => crate::audio::mixer_set_up::close_way(
                mode(),
                &mut crate::audio::restore::game_part,
            )?,
            5 => {
                sd::release_primary(
                    game_addr(PRIMARY_RVA) as u32,
                    game_addr(sd::VOLUME_KEPT_RVA) as u32,
                    &mut sd::game_part,
                )?;
                false
            }
            6 => sd::reopen_sound(
                game_addr(SOUND_MODE_RVA) as u32,
                game_addr(sd::OPENED_MODE_RVA) as u32,
                &mut sd::game_part,
            )?,
            7 => !crate::audio::primary::make_primary(
                &crate::audio::primary::PrimaryVars::game(),
                &mut sd::game_part,
            )?,
            8 => {
                crate::audio::mixer_set_up::open_way(mode(), &mut crate::audio::restore::game_part)?
            }
            9 => {
                crate::system::sleep::sleep(r.eax);
                false
            }
            10 => {
                crate::audio::thread_parts::play_primary(
                    game_addr(PRIMARY_RVA) as u32,
                    game_addr(BUFFER_LOST_RVA) as u32,
                );
                false
            }
            _ => {
                crate::system::restore_window::restore_window();
                false
            }
        })
    }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = set_mode(r, &ModeVars::game(), &mut game_part)?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

// ---- self-test ----

static NOTED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());
static CARRIES: Mutex<[bool; 13]> = Mutex::new([false; 13]);

fn test_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    let eax = if matches!(place, 0 | 1 | 2 | 9 | 11) {
        r.eax
    } else {
        0
    };
    let (ebx, ecx, edx) = if place == 2 {
        (r.ebx, r.ecx, r.edx)
    } else {
        (0, 0, 0)
    };
    NOTED.lock().unwrap().push([place, eax, ebx, ecx, edx]);
    Ok(CARRIES.lock().unwrap()[place as usize])
}

fn handler(place: u32, r: &mut Regs) {
    let carry = test_part(place, r).unwrap_or(false);
    r.set_carry(carry);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // 0 mode, 1 channels, 2 opened; then the caps (0x18 dwords).
    let mut m = vec![0u32; 3 + 0x18];
    let at = m.as_mut_ptr() as u32;
    let v = ModeVars {
        mode: at,
        channel_count: at + 4,
        opened: at + 8,
        caps: at + 12,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A426,
            len: 0x104,
            entry: &[0x60, 0x60, 0x8D, 0x05, 0xCD, 0xA6, 0x4D, 0x00],
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
            (0x4F82BC, v.mode),
            (0x50634C, v.channel_count),
            (0x4F82C0, v.opened),
            (0x506ACC, v.caps + 0x24),
            (0x506AD0, v.caps + 0x28),
            (0x506AE4, v.caps + 0x3C),
            (0x506AE8, v.caps + 0x40),
        ],
        9,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    for mode in [0x11221u32, 0x2441220, 0x511220, 0x1511220, 0x121221] {
        for hardware in [0u32, 4, 20] {
            for failing in [None, Some(6usize), Some(7)] {
                let run = |rust: bool| {
                    unsafe {
                        let p = at as *mut u32;
                        for n in 0..m.len() {
                            p.add(n).write_volatile(0x5555_5555);
                        }
                        let caps = p.add(3);
                        for (n, x) in [
                            (9, hardware),
                            (10, hardware / 2),
                            (15, hardware + 1),
                            (16, hardware / 4),
                        ] {
                            caps.add(n).write_volatile(x);
                        }
                    }
                    {
                        let mut c = CARRIES.lock().unwrap();
                        *c = [false; 13];
                        if let Some(p) = failing {
                            c[p] = true;
                        }
                    }
                    NOTED.lock().unwrap().clear();
                    let before = Regs {
                        eax: mode,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let carry = if rust {
                        set_mode(&before, &v, &mut test_part).ok().map(|ok| !ok)
                    } else {
                        unsafe { call_game(&mut ro, c.addr()) };
                        ro.preserved_except(&before, 0).then(|| ro.carry())
                    };
                    (
                        carry,
                        std::mem::take(&mut *NOTED.lock().unwrap()),
                        unsafe { std::slice::from_raw_parts(at as *const u32, m.len()) }.to_vec(),
                    )
                };
                let mut theirs = run(false);
                // The fix: the streams stopped with action 0.
                for n in &mut theirs.1 {
                    if n[0] == 2 {
                        n[4] = 0;
                    }
                }
                let mine = run(true);
                t.check(theirs.0.is_some() && theirs == mine, || {
                    format!("mode {mode:#x}, hardware {hardware}, failing {failing:?}: original {:?} {:x?} {:x?}; rust {:?} {:x?} {:x?}", theirs.0, theirs.1, &theirs.2[..3], mine.0, mine.1, &mine.2[..3])
                });
            }
        }
    }
    *CARRIES.lock().unwrap() = [false; 13];
    drop(m);
    t.finish()
}
