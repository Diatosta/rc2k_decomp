//! `physics::take_asked_gear` (0x43C1E3): `g_car` +0x238 (`unk_71A210`) 0; the gear
//! asked for, bits 8 .. 11 of +0x468 (`unk_71A440`) as a signed nibble, through
//! `physics::allowed_gear` into `dword_614C40`; then `physics::change_gear`.
//!
//! In:  nothing
//! Out: as `change_gear` leaves them; no flags read (the caller returns)
//!
//! Caller: `sub_43C15C`.
//! Refuses what the two calls refuse; what was written by then is put back.
//! Self-test: the original and the Rust version on the game's variables (the work car
//! and `dword_614C40` saved and put back): asked gears -8 .. 7, gears, bit patterns, the
//! two words of `allowed_gear`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::allowed_gear::allowed_gear;
use crate::physics::change_gear::change_gear;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::take_asked_gear", rva(0x43C1E3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// What the function and its calls write.
fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![(a(0x719FD8), 0xFE0), (a(0x614C40), 4)]
}

/// The Rust version.
pub unsafe fn take_asked_gear() -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let run = || -> Result<Left, Refused> {
        unsafe {
            game_write(rva(0x71A210), 0u32);
            let word: u32 = game_read(rva(0x71A440));
            let asked = (((word & 0xF00) << 20) as i32 >> 28) as u32;
            let gear = allowed_gear(asked)?;
            game_write(rva(0x614C40), gear);
            change_gear()
        }
    };
    let result = run();
    if result.is_err() {
        restore(&all, &saved);
    }
    result
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { take_asked_gear() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = regions();
    let saved = capture(&all);
    let hundred: u32 = unsafe { game_read(rva(0x5C5D14)) };
    for asked in [0u32, 1, 2, 7, 0xF, 0xE, 0x8, 3] {
        for gear in [0u32, 1, 2, u32::MAX] {
            for (locked, open, bits) in [
                (0u32, 1u32, 0u32),
                (0, 1, 0x0AA),
                (hundred, 1, 0),
                (0, 0, 0),
            ] {
                let set = || unsafe {
                    game_write(rva(0x71A440), (0x1234_50FF & !0xF00) | (asked << 8));
                    game_write(rva(0x71A1AC), gear);
                    game_write(rva(0x71A290), locked);
                    game_write(rva(0x71A3CC), open);
                    game_write(rva(0x71A40C), bits);
                    game_write(rva(0x71A018), asked % 2);
                    game_write(rva(0x71A1B0), 3000.0f32.to_bits());
                    game_write(rva(0x71A190), 12.5f32.to_bits());
                    game_write(rva(0x71A194), 13.0f32.to_bits());
                };
                // A gear the Rust version refuses (below -1: the original reads past
                // `g_gearRatios`) is not compared: in the game the original runs there.
                let before = capture(&all);
                set();
                let refused = unsafe { take_asked_gear() }.is_err();
                restore(&all, &before);
                if refused {
                    continue;
                }
                check_state(
                    &mut t,
                    &format!(
                        "asked {asked:#x}, gear {gear:#x}, words {locked:#x} {open}, bits {bits:#x}"
                    ),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    &rust,
                );
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
