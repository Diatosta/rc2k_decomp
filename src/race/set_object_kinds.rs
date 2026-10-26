//! `race::set_object_kinds` (0x425BB4): the object slots' kinds, the objects loaded, the
//! cars' collision points.
//!
//! In:  nothing
//! Out: the registers as the last call leaves them
//!
//! See `race/objects.rs`. One caller, `race::reload_objects`. Refuses more cars than the
//! eight blocks. It begins with its call: manual patch, private copy (its six calls
//! relocated). Shadow mode runs the original alone (it loads). Self-test: a redirected copy
//! whose six calls note themselves, 0, 2 and 8 cars, the game's variables put back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::mud::{copy, run_both, test_part};
use crate::race::objects::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::set_object_kinds", rva(0x425BB4), thunk, &ORIGINAL)
    .private_copy(0xF5)
    .relocate_calls(&[0x00, 0x95, 0xA4, 0xB3, 0xCB, 0xE2])
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
    match unsafe { set_object_kinds(r, &mut game_part) } {
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
        let g = unsafe { set_object_kinds(r, &mut game_part) }?;
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
    let copy = match copy(
        0x425BB4,
        0xF5,
        &[0xE8, 0xFE, 0x0E, 0x00, 0x00],
        &[0x00, 0x95, 0xA4, 0xB3, 0xCB, 0xE2],
        CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        (game_addr(FIXED_KINDS_RVA) as u32, 0x20),
        (game_addr(SLOT_KINDS_RVA) as u32, 0x400),
        at(KEPT_RVA),
    ];
    let mut all = regions.to_vec();
    all.push(at(CARS_RVA));
    let saved = capture(&all);
    for cars in [0u32, 2, 8] {
        run_both(
            &mut t,
            &format!("{cars} cars"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&all, &saved);
                game_write(CARS_RVA, cars);
            },
            &|r| unsafe { set_object_kinds(r, &mut test_part) },
        );
    }
    restore(&all, &saved);
    t.finish()
}
