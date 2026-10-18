//! `physics::place_car_sound` (0x45CC5C): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) placed: +0x224 1; the car's place (`g_car`
//! +0x134 x, +0x138 y, +0x13C z) less +0x6A4 .. +0x6AC (`fild` / `fisub`, exact), turned
//! by `g_viewTurnMatrix` (`scene::rotate_point`), each part times `flt_5C5DA0` (1/65536)
//! into +0x214, +0x218, +0x21C; +0x10 1. All in `pusha` / `popa`. A reading: the car's
//! sound placed against the view (+0x6A4 .. the listener's place).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_441324` (its tail).
//! Refuses (before anything) a null sound object.
//! Self-test: the original and the Rust version on the game's variables (the place, the
//! view matrix and the object table saved and put back) with a test object, under three
//! precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, ext_times, ints_diff_80, with_control_word};
use crate::scene::matrix3::{rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::place_car_sound", rva(0x45CC5C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

const VIEW_TURN: usize = 0x5EA24C; // g_viewTurnMatrix
const OBJECTS: usize = 0x512A10; // g_raceSoundObjects

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn object() -> u32 {
    g(OBJECTS + 4 * (g(0x71B3F8) & 7) as usize)
}

/// The Rust version.
pub unsafe fn place_car_sound() -> Result<Left, Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    unsafe {
        put(o + 0x224, 1);
        let d = |a: usize, b: usize| ints_diff_80(g(a) as i32, g(b) as i32);
        let p = [
            d(0x71A10C, 0x71A67C),
            d(0x71A110, 0x71A680),
            d(0x71A114, 0x71A684),
        ];
        let out = rotate_point(game_addr(rva(VIEW_TURN)) as u32, p);
        let k = g(0x5C5DA0);
        for (n, e) in out.iter().enumerate() {
            put(o + 0x214 + 4 * n as u32, rounded(ext_times(*e, k)));
        }
        put(o + 0x10, 1);
    }
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_car_sound() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let o = object();
    let regions: Vec<Region> = if o == 0 {
        vec![]
    } else {
        vec![(o + 0x10, 4), (o + 0x214, 0x14)]
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut object = vec![0u32; 0x98];
    let o = object.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all: Vec<Region> = vec![
        (o, 0x260),
        (a(0x71B3F8), 4),
        (a(OBJECTS), 0x20),
        (a(0x71A10C), 12),
        (a(0x71A67C), 12),
        (a(VIEW_TURN), 0x40),
    ];
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=6u32 {
                let set = || unsafe {
                    for k in 0..0x98u32 {
                        put(o + 4 * k, 0xA5A5_0000 | k);
                    }
                    game_write(rva(0x71B3F8), seed);
                    for k in 0..8 {
                        game_write(rva(OBJECTS + 4 * k), o);
                    }
                    for k in 0..3usize {
                        game_write(
                            rva(0x71A10C + 4 * k),
                            seed.wrapping_mul(0x0135_7913 + k as u32),
                        );
                        game_write(
                            rva(0x71A67C + 4 * k),
                            (seed + k as u32).wrapping_mul(0x8642_0864),
                        );
                    }
                    let mut f = [0u32; 16];
                    test_floats(seed * 17 + 1, &mut f);
                    for (k, v) in f.iter().enumerate() {
                        game_write(rva(VIEW_TURN + 4 * k), *v);
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    drop(object);
    t.finish()
}
