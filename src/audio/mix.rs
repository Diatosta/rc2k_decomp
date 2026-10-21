//! `audio::mix` (0x40BDC5): a turn of the game's mixer into the primary buffer.
//!
//! In:  nothing
//! Out: every register preserved (the carry of a failure is not read by its caller)
//!
//! `GetCurrentPosition` (+0x10) of `g_primaryBuffer` into `g_playCursor`; the bytes the
//! play cursor moved since the last turn (round the buffer's `g_primarySize`); nothing
//! for none; more than `g_mixMost` (40 ms) are cut to it and the write cursor put that far
//! ahead of the play cursor. Then `audio::mix_channels` for them, `Lock` (+0x2C) from
//! `g_writeCursor` into `g_lockedParts`, the write cursor moved on round the buffer,
//! `audio::write_mix`, `Unlock` (+0x4C). A failing call ends it with the carry, and
//! `DSERR_BUFFERLOST` sets `g_soundBufferLost`. (A `GetStatus` at 0x40BE9D is in it but not
//! reached.) One caller, the mixer's way of the turn.
//! Shadow mode runs the original alone. Self-test: a redirected copy on the test
//! DirectSound whose two calls note eax, 24 cases.

use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::mixer::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::mix", rva(0x40BDC5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

#[derive(Clone, Copy)]
pub struct Vars {
    pub primary: u32,
    pub size: u32,
    pub most: u32,
    pub play: u32,
    pub write: u32,
    pub parts: u32,
    pub lost: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            primary: a(PRIMARY_RVA),
            size: a(PRIMARY_SIZE_RVA),
            most: a(MIX_MOST_RVA),
            play: a(PLAY_CURSOR_RVA),
            write: a(WRITE_CURSOR_RVA),
            parts: a(LOCKED_PARTS_RVA),
            lost: a(BUFFER_LOST_RVA),
        }
    }
}

/// Its two callees: 0 `mix_channels` (eax the bytes), 1 `write_mix`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// True for no failure.
pub unsafe fn mix(v: &Vars, part: Part) -> Result<bool, Refused> {
    unsafe {
        let primary = read(v.primary);
        let failed = |r: i32| {
            if r == BUFFER_LOST {
                write(v.lost, 1);
            }
            false
        };
        let old = read(v.play);
        let position: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x10));
        let r = position(primary, v.play, 0);
        if r != 0 {
            return Ok(failed(r));
        }
        // Converted: `rc2k_core::audio::streams::cursor_moved` and `round`.
        use rc2k_core::audio::streams::{cursor_moved, round};
        let size = read(v.size);
        let Some((bytes, write_at)) = cursor_moved(old, read(v.play), size, read(v.most)) else {
            return Ok(true);
        };
        if let Some(at) = write_at {
            write(v.write, at);
        }
        let mut r = Regs {
            eax: bytes,
            ..Regs::default()
        };
        part(0, &mut r)?;
        let bytes = r.eax;
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x2C));
        let p = v.parts;
        let r = lock(primary, read(v.write), bytes, p, p + 4, p + 8, p + 12, 0);
        if r != 0 {
            return Ok(failed(r));
        }
        write(v.write, round(bytes.wrapping_add(read(v.write)), size));
        part(1, &mut Regs::default())?;
        let [p1, n1, p2, n2] = (p as *const [u32; 4]).read_unaligned();
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x4C));
        let r = unlock(primary, p1, n1, p2, n2);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}

pub fn game_part(which: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if which == 0 {
            crate::audio::mix_channels::mix_channels_in_game(r.eax)
        } else {
            crate::audio::write_mix::write_mix_in_game()
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { mix(&Vars::game(), &mut game_part) } {
        Ok(fine) => r.set_carry(!fine),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push((place, if place == 0 { r.eax } else { 0 }));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let sound = crate::audio::test_sound::TestSound::new();
    let mut vars = Box::new([0u32; 10]);
    let at = vars.as_mut_ptr() as u32;
    let v = Vars {
        primary: at,
        size: at + 4,
        most: at + 8,
        play: at + 12,
        write: at + 16,
        parts: at + 20,
        lost: at + 36,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40BDC5,
            len: 0x107,
            entry: &[0x60, 0x8D, 0x05, 0xE4, 0x68, 0x50, 0x00],
            calls: &[0x5D, 0xAA],
            jumps_out: &[],
        },
        &|callee| {
            [0x40BECC, 0x40C03A]
                .iter()
                .position(|&a| rva(a) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x506A8C, v.primary),
            (0x5068D8, v.size),
            (0x5068DC, v.most),
            (0x5068E4, v.play),
            (0x5068E8, v.write),
            (0x506A98, v.parts),
            (0x506A9C, v.parts + 4),
            (0x506AA0, v.parts + 8),
            (0x506AA4, v.parts + 12),
            (0x4F82CC, v.lost),
        ],
        27,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let mut case = 0;
    for (old, new) in [
        (0x100u32, 0x100u32),
        (0x100, 0x300),
        (0x1F00, 0x100),
        (0x100, 0x1900),
        (0x0, 0x1FFC),
        (0x800, 0x7FC),
    ] {
        for (position, lock, unlock) in [
            (0, 0, 0),
            (BUFFER_LOST, 0, 0),
            (0, -1, 0),
            (0, 0, BUFFER_LOST),
        ] {
            case += 1;
            let results = crate::audio::test_sound::Results {
                cursor: new,
                position,
                lock,
                unlock,
                parts: (0x80, 0x40),
                ..Default::default()
            };
            let run = |rust: bool, vars: &mut [u32; 10]| {
                *vars = [sound.buffer(3), 0x2000, 0x1000, old, 0x1800, 7, 7, 7, 7, 0];
                sound.begin(results);
                NOTED.lock().unwrap().clear();
                let mut ro = Regs::SENTINEL;
                let done = if rust {
                    unsafe {
                        mix(&v, &mut |p, r| {
                            handler(p, r);
                            Ok(())
                        })
                    }
                    .map(|f| (f, true))
                } else {
                    unsafe { call_game(&mut ro, copy.addr()) };
                    Ok((!ro.carry(), ro.preserved_except(&Regs::SENTINEL, 0)))
                };
                (
                    done,
                    // Volatile: the code under test wrote it behind the reference.
                    unsafe { std::ptr::from_mut(vars).read_volatile() },
                    sound.noted(),
                    std::mem::take(&mut *NOTED.lock().unwrap()),
                )
            };
            let theirs = run(false, &mut vars);
            let mine = run(true, &mut vars);
            let ok = matches!(theirs.0, Ok((_, true)))
                && mine.0.is_ok()
                && theirs.1 == mine.1
                && theirs.2 == mine.2
                && theirs.3 == mine.3;
            t.check(ok, || {
                format!(
                    "case {case}: cursor {old:#x} -> {new:#x}, results {position:#x} {lock:#x} {unlock:#x}: original {:x?} {:x?} {:x?}; rust {:x?} {:x?} {:x?}",
                    theirs.1, theirs.2, theirs.3, mine.1, mine.2, mine.3
                )
            });
        }
    }
    t.finish()
}
