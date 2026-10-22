//! `race::start_cars` (0x446463): race init's first step: each car of the race
//! (`g_raceCars`) made current, loaded into the work copy (`drive::load_car`), set at
//! its start (`race::start_car`), the work car's +0xA3C (8 dwords) and +0xA5C (0x11)
//! cleared, stored back (`drive::store_car`).
//!
//! In:  nothing
//! Out: ecx = the count; the rest preserved (`pusha` / `popa` round each car)
//!
//! See `race/car_starts.rs`. One caller, `sub_4462B9` (race init). Refuses no cars (the
//! original's `jnz` would run on) or more than 8, before anything. Shadow mode runs the
//! original alone (`settle_car`'s game calls). Self-test: a copy whose calls go to
//! stand-ins (the car's loading and storing among them: nothing copied), against the Rust
//! version on the same; 1, 2, 5 and 8 cars.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::car_starts::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::start_cars", rva(0x446463), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { start_cars(r, &mut game_part) } {
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
        let g = unsafe { start_cars(r, &mut game_part) }?;
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
        0x446463,
        0x4A,
        &[0xB9, 0x00, 0x00, 0x00, 0x00],
        &[0x0C, 0x11, 0x3A],
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
            cars_2: 1,
            current: 0,
            cars: 2,
        },
        Case {
            mode: 0,
            no_place: 0,
            side: 0,
            cars_2: 1,
            current: 3,
            cars: 5,
        },
        Case {
            mode: 0,
            no_place: 0,
            side: 0,
            cars_2: 1,
            current: 7,
            cars: 8,
        },
    ] {
        both(&mut t, "start_cars", &copy, &case, &|r, part| unsafe {
            start_cars(r, part)
        });
    }
    restore(&regions, &saved);
    t.finish()
}
