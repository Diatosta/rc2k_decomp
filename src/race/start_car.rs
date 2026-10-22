//! `race::start_car` (0x4464AD): the work car set at its start: its checkpoint on the
//! best line (by its number), its place there, back and to the side, the nearest
//! checkpoint and its check count, then `race::reset_car_motion`.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: the registers as `reset_car_motion` leaves them
//!
//! See `race/car_starts.rs`. Callers: `race::start_cars` (race init) and `sub_43B635`
//! (0x43BA88, stream B's). Refuses a nearest checkpoint past the lists (when met; the
//! original reads past `g_checkCounts`). Shadow mode runs the original alone
//! (`settle_car`'s game calls). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version on the same: every branch of `dword_62389C` (0 .. 4), `dword_64BB18`,
//! `dword_52A910`, `dword_63DB94`, odd and even cars.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::car_starts::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::start_car", rva(0x4464AD), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { start_car(r, &mut game_part) } {
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
        let g = unsafe { start_car(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x4464AD,
        0x1C2,
        &[0xC7, 0x05, 0x50, 0xA4, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x99, 0xE6, 0x104, 0x14E, 0x15F, 0x19E],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    for case in [
        Case {
            mode: 0,
            no_place: 0,
            side: 0,
            cars_2: 1,
            current: 0,
            cars: 1,
        },
        Case {
            mode: 0,
            no_place: 0,
            side: 0,
            cars_2: 2,
            current: 1,
            cars: 2,
        },
        Case {
            mode: 0,
            no_place: 0,
            side: 1,
            cars_2: 1,
            current: 2,
            cars: 3,
        },
        Case {
            mode: 1,
            no_place: 0,
            side: 0,
            cars_2: 2,
            current: 3,
            cars: 4,
        },
        Case {
            mode: 2,
            no_place: 0,
            side: 1,
            cars_2: 2,
            current: 5,
            cars: 6,
        },
        Case {
            mode: 3,
            no_place: 0,
            side: 0,
            cars_2: 2,
            current: 7,
            cars: 8,
        },
        Case {
            mode: 4,
            no_place: 0,
            side: 1,
            cars_2: 2,
            current: 1,
            cars: 2,
        },
        Case {
            mode: 0,
            no_place: 1,
            side: 1,
            cars_2: 2,
            current: 4,
            cars: 5,
        },
        Case {
            mode: 2,
            no_place: 2,
            side: 0,
            cars_2: 1,
            current: 6,
            cars: 7,
        },
    ] {
        both(&mut t, "start_car", &copy, &case, &|r, part| unsafe {
            start_car(r, part)
        });
    }
    restore(&regions, &saved);
    t.finish()
}
