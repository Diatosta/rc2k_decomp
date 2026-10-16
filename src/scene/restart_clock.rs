//! `scene::restart_clock` (0x450381): `g_sceneClock` = the tick count.
//!
//! In:  nothing
//! Out: eax = the tick count; every other register preserved
//!
//! 21 callers (`race::load_stage` and the race's code). Manual patch with a private copy:
//! it begins with its call. Shadow mode runs the original alone (the clock moves).
//! Self-test: a redirected copy whose `system::tick_count` gives the test's times.

use crate::framework::{
    EAX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::restart_clock", rva(0x450381), thunk, &ORIGINAL)
    .private_copy(0xB)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock

/// The clock set to `now`; eax.
pub fn restart_clock(now: u32) -> u32 {
    unsafe { game_write(CLOCK_RVA, now) };
    now
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    r.eax = restart_clock(crate::system::tick_count::tick_count());
}

static NOW: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    r.eax = NOW.load(Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x450381)),
            0xB,
            &[0xE8, 0x9E, 0x0C, 0xFC, 0xFF],
            &[0x00],
            &|_| Some(stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let var = [at(CLOCK_RVA)];
    let saved = capture(&var);
    for now in [0u32, 0x1234_5678, u32::MAX] {
        NOW.store(now, Ordering::SeqCst);
        unsafe { game_write(CLOCK_RVA, 0xAAAA_AAAAu32) };
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (ro.preserved_except(&before, EAX), ro.eax, unsafe {
            game_read::<u32>(CLOCK_RVA)
        });
        unsafe { game_write(CLOCK_RVA, 0xAAAA_AAAAu32) };
        let eax = restart_clock(now);
        let mine = (true, eax, unsafe { game_read::<u32>(CLOCK_RVA) });
        t.check(theirs == mine, || {
            format!("now {now:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&var, &saved);
    t.finish()
}
