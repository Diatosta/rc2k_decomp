//! `physics::false_start` (0x43B9B4): the work car's start held back (`g_car` +0x14 ..
//! +0x1C, a reading: a jump start's penalty). +0x18 (`dword_719FF0`) is set once the car
//! is in gear; while it is set and +0x1C (`dword_719FF4`) is 0, a timer +0x14
//! (`dword_719FEC`, in eax) below 0x320 (by the sign of the difference, `jns`) starts the
//! count +0x1C at 500, with +0x71C (`dword_71A6F4`) 0xCC, +0x730 0 and +0x72C -1 (the
//! notice block, by its look). Each call the count goes down; on reaching 0: +0x88
//! (`dword_71A060`) + 1000, +0x14 and +0x18 0, +0x730 0, +0x72C 300, +0x724 0, +0x71C
//! 0xCA, +0x720 0, +0x728 0, and `race::start_car` (the car put back at its start).
//!
//! In:  nothing
//! Out: eax the timer when it was compared; when `start_car` ran, the registers as it
//!      leaves them; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43B635` (0x43B9A9), every physics step.
//! Refuses what `start_car` refuses. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose call goes to the stand-ins, against the Rust version with the
//! same: the set flag, gear, count and timer across their branches; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::physics::reset_parts::{both, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{capture, restore};
use rc2k_core::physics::notices::Notice;
use rc2k_core::physics::start::CarStart;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::false_start", rva(0x43B9B4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TIMER: usize = rva(0x719FEC); // g_car + 0x14
const HELD: usize = rva(0x719FF0); // + 0x18
const COUNT: usize = rva(0x719FF4); // + 0x1C
const GEAR: usize = rva(0x71A1AC); // g_carGear

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

/// The Rust version, `start_car` through `part` (place 11).
// Converted: `rc2k_core::physics::start::CarStart::false_start`.
pub unsafe fn false_start(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let was = CarStart {
        timer: g(TIMER),
        held: g(HELD),
        count: g(COUNT),
        time: g(rva(0x71A060)),
        ..CarStart::default()
    };
    let notice_was = Notice {
        id: g(rva(0x71A6F4)),
        w6f8: g(rva(0x71A6F8)),
        last: g(rva(0x71A6FC)),
        shown: g(rva(0x71A700)),
        time: g(rva(0x71A704)),
        w708: g(rva(0x71A708)),
    };
    let (mut car, mut notice) = (was, notice_was);
    let done = car.false_start(g(GEAR), &mut notice);
    if let Some(timer) = done.timer {
        r.eax = timer;
    }
    for (at, before, now) in [
        (HELD, was.held, car.held),
        (COUNT, was.count, car.count),
        (rva(0x71A060), was.time, car.time),
        (TIMER, was.timer, car.timer),
        (rva(0x71A6F4), notice_was.id, notice.id),
        (rva(0x71A6F8), notice_was.w6f8, notice.w6f8),
        (rva(0x71A6FC), notice_was.last, notice.last),
        (rva(0x71A700), notice_was.shown, notice.shown),
        (rva(0x71A704), notice_was.time, notice.time),
        (rva(0x71A708), notice_was.w708, notice.w708),
    ] {
        if now != before {
            p(at, now);
        }
    }
    if done.put_back {
        part(11, &mut r)?;
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, false_start);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43B9B4,
        0xDA,
        &[0x83, 0x3D, 0xF0, 0x9F, 0x71, 0x00, 0x00],
        &[0xD4],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = regions();
    let saved = capture(&all);
    let mut n = 0u32;
    for held in [0u32, 1] {
        for gear in [0u32, 2] {
            for count in [0u32, 1, 2, 500] {
                for timer in [0x31Fu32, 0x320, 0xFFFF_FFFB, 0x8000_0000] {
                    n += 1;
                    let fill = || {
                        prepare(
                            &Case {
                                mode: n % 4,
                                no_place: 0,
                                side: 0,
                                cars_2: 1,
                                current: n % 4,
                                cars: 4,
                            },
                            n.wrapping_mul(0x2026_1017),
                        );
                        p(HELD, held);
                        p(GEAR, gear);
                        p(COUNT, count);
                        p(TIMER, timer);
                    };
                    both(
                        &mut t,
                        &format!("held {held}, gear {gear}, count {count}, timer {timer:#x}"),
                        &copy,
                        0,
                        Regs::SENTINEL,
                        &fill,
                        false_start,
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
