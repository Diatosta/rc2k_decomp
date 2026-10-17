//! `race::reset_car_motion` (0x43FB47): `race::settle_car`, `race::invert_car_matrix`,
//! then the work car's motion cleared and its angles kept as the start's.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: eax = the work car's +0x68C, less one when not 0; the rest as the calls leave
//!      them
//!
//! See `race/car_starts.rs`. Callers: `race::start_car` and three more (0x429492,
//! 0x43768C, 0x43FB23). It begins with its call: private copy, both calls relocated,
//! manual patch. Shadow mode runs the original alone (`settle_car`'s game calls).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::car_starts::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::reset_car_motion", rva(0x43FB47), thunk, &ORIGINAL)
    .private_copy(0xEB)
    .relocate_calls(&[0x00, 0x05])
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
    match unsafe { reset_car_motion(r, &mut game_part) } {
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43FB47,
        0xEB,
        &[0xE8, 0xE6, 0x00, 0x00, 0x00],
        &[0x00, 0x05],
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
            cars_2: 0,
            current: 0,
            cars: 1,
        },
        Case {
            mode: 2,
            no_place: 1,
            side: 1,
            cars_2: 1,
            current: 5,
            cars: 6,
        },
    ] {
        both(
            &mut t,
            "reset_car_motion",
            &copy,
            &case,
            &|r, part| unsafe { reset_car_motion(r, part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
