//! `race::set_up_mode_2_cars` (0x43613A): `sub_43603C`'s arcade set-up (mode 2): the
//! rally's car count, best and start places, one player and four cars, the arcade cars'
//! types and numbers from the noise seed (`dword_60519C`, shuffled), the race tables
//! cleared, four automatic set-ups, the player's +0x3C / +0x658 .. +0x66C; logged.
//!
//! In:  nothing
//! Out: eax = the cars behind (count - start place), ecx = the count; the rest as the
//!      steps leave them
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_cars`. Refuses a car count of 0
//! or more than 89 (the original's `jnz` runs on / past `dword_604ECC`), before
//! anything. Shadow mode runs the original alone (it logs). Self-test: a copy whose
//! calls note themselves (`scene::next_noise` the real one, from the same seed), against
//! the Rust version on the same: each rally, start place by `dword_60101C` and the
//! stage, the players' numbers in the way; the refusal on the Rust version alone.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: &[usize] = &[
    0x07, 0x14, 0x20, 0x2D, 0x39, 0x9E, 0xAA, 0xB7, 0xC3, 0xD0, 0xDC, 0x100, 0x105, 0x146, 0x181,
    0x1BA, 0x1E5, 0x1F0, 0x1FB, 0x206, 0x211, 0x287, 0x293, 0x2A0, 0x2B3, 0x2C2,
];

pub static HOOK: RawHook =
    RawHook::new("race::set_up_mode_2_cars", rva(0x43613A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up_mode_2_cars(r, &mut cars_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The stand-ins' part, but `scene::next_noise` (place 18) the real one, as the copy's.
fn part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if place == 18 {
        cars_part(place, r)
    } else {
        crate::race::mud::test_part(place, r)
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::race::mud::copy(
        0x43613A,
        0x2C8,
        &[0x60, 0x8D, 0x05, 0xF4, 0xB9, 0x4D, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = arcade_regions();
    let saved = capture(&regions);
    let _ = game_write::<u32> as unsafe fn(usize, u32);
    for (rally, stage_in, kind, seed, players, taken) in [
        (0u32, 0u32, 0u32, 0x1234_5678u32, 1u32, [0u32, 0u32]),
        (1, 2, 2, 0xDEAD_BEEF, 1, [1, 3]),
        (2, 3, 0, 0, 2, [2, 4]),
        (3, 1, 1, 0x8000_0001, 2, [5, 1]),
        (4, 5, 0, 77, 1, [9, 9]),
        (5, 0, 2, 0xFFFF_FFFF, 1, [1, 2]),
    ] {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!(
                "rally {rally}, stage {stage_in}, kind {kind}, seed {seed:#x}, players {players}"
            ),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| {
                prepare(seed, 2, players, 4, [0, 0]);
                prepare_set_up(0x22, 0, [40, 0]);
                prepare_arcade(rally, stage_in, kind, seed, taken);
            },
            &|r| unsafe { set_up_mode_2_cars(r, &mut part) },
        );
    }
    // Rally 12's count is `dword_60469C`[0], a float's bits: far past 89.
    prepare_arcade(12, 0, 0, 0, [0, 0]);
    let refused = unsafe { set_up_mode_2_cars(&Regs::SENTINEL, &mut part) }.is_err();
    t.check(refused, || {
        "rally 12 (a count past 89): not refused".to_string()
    });
    restore(&regions, &saved);
    t.finish()
}
