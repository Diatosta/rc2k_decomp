//! The car's sound objects placed (IDA 2026-10-18): four functions under the sound
//! drive that, when the current car is listed (`dword_641680[g_currentCar]` not -1), take
//! a band (`physics::sound_band_1` .. `_4`), set two words of the car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) and its place: the car's place (`g_car`
//! +0x134 .. +0x13C, with an offset for one) less the listener's (+0x6A4 ..), `fild` /
//! `fiadd` / `fisub` (exact), turned by `g_viewTurnMatrix`, each part x `flt_5C5DA0`
//! (1/65536); then +0x10 1. All in `pusha` / `popa`.
//!
//! This file: the shared steps and the test of the four.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, ext_times, with_control_word};
use crate::scene::matrix3::{rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};

pub const CURRENT: usize = 0x71B3F8; // g_currentCar
pub const LISTED: usize = 0x641680; // dword_641680
pub const OBJECTS: usize = 0x512A10; // g_raceSoundObjects
pub const VIEW_TURN: usize = 0x5EA24C; // g_viewTurnMatrix

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// Whether car `car` (a number) is listed; refused past 7.
pub fn listed(car: u32) -> Result<bool, Refused> {
    if car > 7 {
        return Err(BadInput("a car number past 7").into());
    }
    Ok(g(LISTED + 4 * car as usize) != u32::MAX)
}

/// The current car's sound object; refused when null.
pub fn object() -> Result<u32, Refused> {
    let o = g(OBJECTS + 4 * (g(CURRENT) & 7) as usize);
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    Ok(o)
}

/// The place (with `offset` added to the car's, when given), turned and scaled, into
/// the object at +`at` .. +`at` + 8.
pub unsafe fn place(o: u32, at: u32, offset: Option<[u32; 3]>) {
    // The original's own steps (`fild` the offset, `fiadd` the car's, `fisub` the
    // listener's; without an offset `fild` the car's, `fisub`): each rounds to the
    // control word's precision, so a big difference keeps fewer bits at 24.
    let part = |k: usize| {
        let car = g(0x71A10C + 4 * k);
        let listener = g(0x71A67C + 4 * k);
        match offset {
            Some(o) => offset_part(o[k], car, listener),
            None => car_part(car, listener),
        }
    };
    unsafe {
        let out = rotate_point(
            game_addr(rva(VIEW_TURN)) as u32,
            [part(0), part(1), part(2)],
        );
        let k = g(0x5C5DA0);
        for (n, e) in out.iter().enumerate() {
            put(o + at + 4 * n as u32, rounded(ext_times(*e, k)));
        }
    }
}

/// `fild car; fisub listener`, as an 80-bit value.
fn car_part(car: u32, listener: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let v = [car, listener];
    crate::math::x87::x87!("fild dword ptr [{v}]", "fisub dword ptr [{v} + 4]", "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// `fild offset; fiadd car; fisub listener`, as an 80-bit value.
fn offset_part(offset: u32, car: u32, listener: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    let v = [offset, car, listener];
    crate::math::x87::x87!("fild dword ptr [{v}]", "fiadd dword ptr [{v} + 4]", "fisub dword ptr [{v} + 8]",
        "fstp tbyte ptr [{o}]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    out
}

/// What a test sets and the functions write: the test object, the tables, the places,
/// the view matrix, the band inputs.
pub fn test_regions(object: u32) -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (object, 0x260),
        (a(CURRENT), 4),
        (a(LISTED), 0x20),
        (a(OBJECTS), 0x20),
        (a(0x71A10C), 12),
        (a(0x71A67C), 12),
        (a(VIEW_TURN), 0x40),
        (a(0x6D1AE0), 0xC),
        (a(0x6D1AEC), 4),
        (a(0x6D3108), 4),
        (a(0x6D3CD0), 8),
        (a(0x71A19C), 4),
        (a(0x6D1AD4), 4),
        (a(0x6D3B4C), 12),
        (a(0x6D321C), 4),
        (a(0x6D1A90), 0x40),
    ]
}

/// The test of one of them: the original and the Rust version on the game's variables
/// (saved and put back) with a test object; `set` adds the function's own inputs.
pub fn test_sound(
    hook: &RawHook,
    original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
    set: &dyn Fn(u32),
    extra: &[Region],
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 0x98];
    let o = cells.as_mut_ptr() as u32;
    let mut all = test_regions(o);
    all.extend_from_slice(extra);
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..8u32 {
                let prepare = || unsafe {
                    for k in 0..0x98u32 {
                        put(o + 4 * k, 0xA5A5_0000 | k);
                    }
                    game_write(rva(CURRENT), seed % 8);
                    for k in 0..8usize {
                        game_write(rva(OBJECTS + 4 * k), o);
                        let v = if (k as u32 + seed).is_multiple_of(3) {
                            u32::MAX
                        } else {
                            k as u32
                        };
                        game_write(rva(LISTED + 4 * k), v);
                    }
                    let mut f = [0u32; 24];
                    test_floats(seed * 19 + 3, &mut f);
                    for (k, v) in f.iter().take(16).enumerate() {
                        game_write(rva(VIEW_TURN + 4 * k), *v);
                    }
                    for k in 0..3usize {
                        game_write(
                            rva(0x71A10C + 4 * k),
                            seed.wrapping_mul(0x0246_8ACE) ^ k as u32,
                        );
                        game_write(
                            rva(0x71A67C + 4 * k),
                            (seed + 3).wrapping_mul(0x1357_9BDF) >> 2,
                        );
                    }
                    // The bands' inputs: values about 0 .. 0x50 by the factors 1.
                    for v in [0x6D1AE0usize, 0x6D1AE4, 0x6D1AE8] {
                        game_write(rva(v), 1.0f32.to_bits());
                    }
                    let band = |n: u32| ((seed * 7 + n * 11) % 0x50) as f32;
                    game_write(rva(0x6D3108), band(1).to_bits());
                    game_write(rva(0x6D3CD0), band(2).to_bits());
                    game_write(rva(0x71A19C), band(3).to_bits());
                    set(seed);
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    original,
                    Regs::SENTINEL,
                    &prepare,
                    &all,
                    rust,
                );
            }
        });
    }
    restore(&all, &saved);
    drop(cells);
    t.finish()
}
