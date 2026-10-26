//! `race::settle_car` (0x43FC32): the work car settled on the ground: pitch and roll
//! found by halving steps against the wheels' heights, then the car lowered onto them.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: eax = wheel 3's height, edx = the lowest; the rest as the calls leave them
//!
//! See `race/car_starts.rs`. One caller, `race::reset_car_motion`. It calls
//! `physics::set_handling` (0x43EB5D) and `physics::probe_ground` (0x45D305), stream B's
//! Rust versions, and `race::get_mat_crash`. It begins with its call: private copy, five calls relocated,
//! manual patch. Shadow mode runs the original alone (the calls change more than can be
//! put back). Self-test: a copy whose calls go to stand-ins that note them and
//! give wheel heights of the test's, against the Rust version on the same stand-ins.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::car_starts::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::settle_car", rva(0x43FC32), thunk, &ORIGINAL)
    .private_copy(0x13F)
    .relocate_calls(&[0x00, 0x3D, 0x42, 0xCD, 0xD2])
    .manual_patch()
    .self_test(self_test);

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
    match unsafe { settle_car(r, &mut game_part) } {
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
        let g = unsafe { settle_car(r, &mut game_part) }?;
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
        0x43FC32,
        0x13F,
        &[0xE8, 0x26, 0xEF, 0xFF, 0xFF],
        &[0x00, 0x3D, 0x42, 0xCD, 0xD2],
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
            mode: 1,
            no_place: 0,
            side: 0,
            cars_2: 0,
            current: 3,
            cars: 2,
        },
    ] {
        both(&mut t, "settle_car", &copy, &case, &|r, part| unsafe {
            settle_car(r, part)
        });
    }
    restore(&regions, &saved);
    t.finish()
}
