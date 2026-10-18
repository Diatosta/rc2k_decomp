//! `math::seed_random` (0x401864): how the random numbers start.
//!
//! In:  nothing
//! Out: every register preserved
//! One step of the generator on the start's time (`dword_5183A4`: hour, minute, second,
//! hundredths) with its date mixed in (`dword_5183A0`: day, month, year); unless that is
//! 0, it is turned left by the day of the week (`byte_51839F`) plus 8 and becomes the
//! state. One caller, `start`, right after the time is taken.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::next_random::{STATE_RVA, TEST_STATES, random_step};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::seed_random", rva(0x401864), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const WEEKDAY_RVA: usize = rva(0x51839F); // byte_51839F
const DATE_RVA: usize = rva(0x5183A0); // dword_5183A0
const TIME_RVA: usize = rva(0x5183A4); // dword_5183A4

// ---- Rust version ----

pub unsafe fn seed_random() {
    crate::math::next_random::wait_for_random();
    unsafe {
        let seed = random_step(game_read(TIME_RVA), game_read(DATE_RVA));
        if seed != 0 {
            let turns = game_read::<u8>(WEEKDAY_RVA).wrapping_add(8) as u32;
            game_write(STATE_RVA, seed.rotate_left(turns & 31));
        }
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { seed_random() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    // Waits while a test owns the generator (`next_random::own_random`).
    crate::math::next_random::wait_for_random();
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: [Region; 2] = [at(STATE_RVA), (game_addr(WEEKDAY_RVA) as u32, 9)];
    let saved = capture(&all);
    for time in TEST_STATES {
        for date in [0u32, 0x07EA_0A0C, 0xFFFF_FFFF, time.rotate_right(1)] {
            for weekday in [0u8, 3, 6, 24, 0xFF] {
                check_state(
                    &mut t,
                    &format!("time={time:#x} date={date:#x} weekday={weekday}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        game_write(STATE_RVA, 0xAAAA_AAAAu32);
                        game_write(WEEKDAY_RVA, weekday);
                        game_write(DATE_RVA, date);
                        game_write(TIME_RVA, time);
                    },
                    &[at(STATE_RVA)],
                    &rust,
                );
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
