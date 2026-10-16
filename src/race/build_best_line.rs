//! `race::build_best_line` (0x445772): the stage's best line built, step by step, with its
//! log.
//!
//! In:  nothing
//! Out: the registers as the last step leaves them
//!
//! See `race/best_line.rs`. Callers: `race::locate_current_course` and `sub_4462B9` (race
//! init). Shadow mode runs the original alone (it logs). Self-test: a redirected copy
//! whose 27 calls note themselves, with two line faces and with one, against the Rust
//! version noting the same.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::best_line::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::build_best_line", rva(0x445772), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { build_best_line(r, &mut build_part) } {
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

/// The offsets of its calls.
const CALLS: &[usize] = &[
    0x07, 0x0D, 0x26, 0x2C, 0x38, 0x3E, 0x4A, 0x50, 0x5C, 0x62, 0x6E, 0x74, 0x80, 0x86, 0x92, 0x98,
    0x9D, 0xA9, 0xB6, 0xC2, 0xCF, 0xDB, 0xE8, 0xF4, 0x101, 0x10D, 0x11A,
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x445772,
        0x135,
        &[0x60, 0x8D, 0x05, 0xA8, 0xC4, 0x4D, 0x00],
        CALLS,
        BUILD_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let watched: Vec<Region> = [FOUND_RVA, rva(0x6365E4), rva(0x6365EC)].map(at).to_vec();
    let mut all = watched.clone();
    all.extend([at(DIRECTION_RVA), at(CHECK_COUNT_RVA), at(LAST_CHECK_RVA)]);
    let saved = capture(&all);
    for found in [5u32, 1] {
        run_both(
            &mut t,
            &format!("{found} line faces"),
            &copy,
            &watched,
            &|| unsafe {
                restore(&all, &saved);
                game_write(FOUND_RVA, found);
                game_write(rva(0x6365E4), 0x55u32);
                game_write(rva(0x6365EC), 0x66u32);
            },
            &|r| unsafe { build_best_line(r, &mut test_part) },
        );
    }
    restore(&all, &saved);
    t.finish()
}
