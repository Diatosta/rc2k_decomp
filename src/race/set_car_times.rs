//! `race::set_car_times` (0x4377D0): the stage's car times (arcade, mode 2): the first
//! car's kind factor (`flt_601C64`), the stage's difficulty and time percentage, the
//! extra time (`dword_601C50`) added to the first number, every number of
//! `g_stageCarNumbers` (`dword_601830` of them) times the factor; all logged.
//!
//! In:  nothing
//! Out: eax = the extra time, ebx = the count (without a stage, 0x4F or more: eax the
//!      stage); the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_cars` (mode 2). Refuses, for a
//! stage below 0x4F, a count of 0 or more than 0x100 (the original's `jnz` runs on /
//! past the numbers), before anything. Shadow mode runs the original alone (it logs).
//! Self-test: a copy whose log calls note themselves, against the Rust version on the
//! same: stages in each rally, below 0x15 and past 0x4E, kinds round 0 and 0x16, counts
//! 1 and 0x100; the refusal on the Rust version alone.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: &[usize] = &[
    0x97, 0xA3, 0xB0, 0xB7, 0xCB, 0xD2, 0xFC, 0x108, 0x115, 0x121, 0x13E, 0x147, 0x154, 0x162,
    0x183, 0x191,
];

pub static HOOK: RawHook =
    RawHook::new("race::set_car_times", rva(0x4377D0), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { set_car_times(r, &mut cars_part) } {
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_car_times(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
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
        0x4377D0,
        0x1A1,
        &[0xA1, 0x58, 0x38, 0x62, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = arcade_regions();
    let saved = capture(&regions);
    let _ = game_write::<u32> as unsafe fn(usize, u32);
    for (stage, kind, count, total) in [
        (0x15u32, 0i32, 3u32, 1000i32),
        (0x1C, 5, 1, 777),
        (0x1D, -3, 8, -500),
        (0x30, 0x16, 0x10, 0x7FFF),
        (0x3A, 0x40, 0x100, 12345),
        (0x44, 2, 5, 0),
        (0x4E, 1, 2, 99),
        (0x4F, 1, 2, 99),
        (0x10, 1, 2, 99),
        (0x99, 3, 0, 1),
    ] {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!("stage {stage:#x}, kind {kind}, count {count}, total {total}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| unsafe {
                prepare(0x1111_0000 ^ stage, 2, 1, 4, [0, 0]);
                prepare_set_up(stage, 0, [0, 0]);
                game_write(CAR_KINDS_RVA, kind);
                game_write(CHECK_POSITS_RVA, count);
                game_write(STAGE_TOTAL_RVA, total);
                for k in 0..MOST_NUMBERS {
                    game_write(SPEEDS_RVA + 4 * k, (k as u32 * 37) ^ 0x155);
                }
            },
            &|r| unsafe { set_car_times(r, &mut part) },
        );
    }
    unsafe {
        prepare_set_up(0x22, 0, [0, 0]);
        game_write(CHECK_POSITS_RVA, 0u32);
    }
    let refused = unsafe { set_car_times(&Regs::SENTINEL, &mut part) }.is_err();
    t.check(refused, || "count 0: not refused".to_string());
    restore(&regions, &saved);
    t.finish()
}
