//! `race::init_mud` (0x449DD6): the mud set up (`render3d initmud`).
//!
//! In:  nothing
//! Out: the registers as `race::sample_mud_colours` leaves them
//!
//! See `race/mud.rs`. One caller, `race::loadlevel_init`. Shadow mode runs the original
//! alone (it takes memory). Self-test: a redirected copy whose four calls note themselves,
//! the game's variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::level_init::GIVES;
use crate::race::mud::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::init_mud", rva(0x449DD6), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { init_mud(r, &mut game_part) } {
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
    let copy = match copy(
        0x449DD6,
        0x2B,
        &[0xB8, 0x00, 0x00, 0x01, 0x00],
        &[0x11, 0x1B, 0x20, 0x25],
        &[(0x4028A7, 0), (0x449D95, 4), (0x449E01, 5), (0x44A0D6, 6)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [at(MUD_RVA), at(LIST_TOTAL_RVA)];
    let saved = capture(&regions);
    for total in [0u32, 0x10_0000] {
        run_both(
            &mut t,
            &format!("total {total:#x}"),
            &copy,
            &regions,
            &|| unsafe {
                GIVES.store(0x0BAD_0000, Ordering::SeqCst);
                game_write(LIST_TOTAL_RVA, total);
            },
            &|r| unsafe { init_mud(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
