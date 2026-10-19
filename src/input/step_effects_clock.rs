//! `input::step_effects_clock` (0x40F257): the force feedback's clock: every 40 ms the
//! effects brought up to date.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! `system::tick_count` less the last time (`dword_518084`); under 40 ms (`js`)
//! nothing. Else the last time = now, `dword_518088` = 1000 / now held to 1000 (unsigned
//! `div`, as a float: the original divides by the clock, not by the time passed, so it
//! is 1.0 once the clock is past a second; notes' bugs), `dword_51808C` counts up, and
//! `input::update_effects` (its Rust version in the game).
//! One caller, `sound::race_turn`.
//! Refuses (before anything) a clock of 0 when due (the original's `div` traps).
//! Shadow mode runs the original alone (the clock moves, the effects are sent).
//! Self-test: a copy whose two calls go to stand-ins (the test's clock), against the
//! Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87::from_int;
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{
    EFFECTS, Part, TEST_TICK, TICK, both, copy_of, dispatch_turn, run_copy,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::step_effects_clock", rva(0x40F257), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x46;
const OUTS: [usize; 2] = [0x07, 0x3D];
const ENTRY: [u8; 7] = [0x60, 0x8B, 0x1D, 0x84, 0x80, 0x51, 0x00];
/// dword_518084 the last time, dword_518088 the rate, dword_51808C the steps counted.
const LAST: usize = 0x518084;
const RATE: usize = 0x518088;
const STEPS: usize = 0x51808C;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

// ---- Rust version ----

pub fn step_effects_clock_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut c = *r;
    c.ebx = g(LAST);
    part(TICK, &mut c)?;
    c.eax = c.eax.wrapping_sub(c.ebx);
    if (c.eax.wrapping_sub(0x28) as i32) < 0 {
        return Ok(());
    }
    c.ebx = c.ebx.wrapping_add(c.eax);
    let divisor = c.ebx.min(1000);
    if divisor == 0 {
        return Err(BadInput("a clock of 0: the original divides by it").into());
    }
    p(LAST, c.ebx);
    (c.eax, c.edx, c.ebx) = (1000 / divisor, 1000 % divisor, divisor);
    p(RATE, from_int(c.eax as i32));
    p(STEPS, g(STEPS).wrapping_add(1));
    part(EFFECTS, &mut c)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, step_effects_clock_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40F257, LEN, &ENTRY, &OUTS, &[TICK, EFFECTS]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all: [Region; 1] = [(game_addr(rva(LAST)) as u32, 12)];
    let saved = capture(&all);
    for (last, now) in [
        (0u32, 39u32),
        (0, 40),
        (100, 500),
        (5000, 5039),
        (5000, 9000),
        (0xFFFF_FFF0, 0x30),
        (0x10, 0),
        (0xFFFF_FFC0, 0),
    ] {
        let fill = || {
            p(LAST, last);
            p(RATE, 0x1234);
            p(STEPS, 7);
            TEST_TICK.store(now, Ordering::SeqCst);
        };
        both(
            &mut t,
            &format!("last {last:#x}, now {now:#x}"),
            &run,
            Regs::SENTINEL,
            &fill,
            &all,
            step_effects_clock_in,
            now == 0 && (now.wrapping_sub(last).wrapping_sub(0x28) as i32) >= 0,
        );
    }
    restore(&all, &saved);
    t.finish()
}
