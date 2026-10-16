//! `race::locate_current_course` (0x4394CB): the stage's course data loaded and set up
//! (`LOADLEVEL_INIT : locacurrcourse`).
//!
//! In:  nothing
//! Out: the registers as `race::build_best_line` leaves them
//!
//! See `race/course_data.rs`. One caller, `sub_44426F` (loadlevel_init). Shadow mode runs
//! the original alone (it reads and logs). Self-test: a redirected copy whose four calls
//! note themselves, against the Rust version noting the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_data::*;
use crate::service::harness::at;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::locate_current_course",
    rva(0x4394CB),
    thunk,
    &ORIGINAL,
)
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
    match unsafe { locate_current_course(r, &mut locate_part) } {
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
    use crate::race::mud::{copy, run_both, test_part};
    use crate::service::harness::{capture, restore};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4394CB,
        0x2D,
        &[0xC7, 0x05, 0x80, 0x93, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x0F, 0x1D, 0x22, 0x27],
        LOCATE_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let watched = [at(rva(0x609380))];
    let saved = capture(&watched);
    run_both(
        &mut t,
        "the calls",
        &copy,
        &watched,
        &|| unsafe { crate::framework::game_write(rva(0x609380), 0x77u32) },
        &|r| unsafe { locate_current_course(r, &mut test_part) },
    );
    restore(&watched, &saved);
    t.finish()
}
