//! `race::load_mud` (0x449D95): the car's mud file read into the mud.
//!
//! In:  nothing
//! Out: eax, edx the file number's quotient and remainder by 10 (their low bytes + '0'),
//!      ebx 10, esi the mud; the rest preserved
//!
//! See `race/mud.rs`. One caller, `race::init_mud`. Self-test: a redirected copy whose read
//! notes itself, cars of the test's, the game's path and variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::mud::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_mud", rva(0x449D95), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { load_mud(r, &mut game_part) } {
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
        0x449D95,
        0x41,
        &[0xA1, 0x10, 0xF4, 0x70, 0x00],
        &[0x05, 0x3A],
        &[(0x421B54, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        (game_addr(MUD_PATH_DIGITS_RVA) as u32, 2),
        at(CAR_RVA),
        at(MUD_RVA),
    ];
    let saved = capture(&regions);
    for car in [0u32, 6, 0x1F, 0x21, 0x40, u32::MAX] {
        run_both(
            &mut t,
            &format!("car {car:#x}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(CAR_RVA, car);
                game_write(MUD_RVA, 0x1234_5678u32);
            },
            &|r| unsafe { load_mud(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
