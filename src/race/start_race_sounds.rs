//! `race::start_race_sounds` (0x40D592): the race's sounds started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sounds.rs`. Callers: `race::wake_car_sounds` and 0x41D855. Refuses a sound
//! object missing. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! two calls note themselves, the game's objects' flags, `dword_514F7C` over 0 .. 9 and -1,
//! put back afterwards.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::race::sounds::*;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::start_race_sounds", rva(0x40D592), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { start_race_sounds(r, &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

static CALLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(place: u32, _: &mut Regs) {
    CALLED.lock().unwrap().push(place);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !objects_there() {
        return t.skip("a race sound object missing");
    }
    // sound::start_race_clock, sound::let_sounds_play.
    let callees = [rva(0x40F1E9), rva(0x409B76)];
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40D592)),
            0x61,
            &[0x60, 0xE8, 0x51, 0x1C, 0x00, 0x00],
            &[0x01, 0x50],
            &|c| {
                callees
                    .iter()
                    .position(|&k| k == c)
                    .map(|n| stand_in(n as u32))
            },
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let regions = regions();
    let saved = capture(&regions);
    for count in [0u32, 1, 3, 8, 9, u32::MAX] {
        let run = |rust: bool| {
            restore(&regions, &saved);
            unsafe { game_write(COUNT_RVA, count) };
            for &(at, _) in &regions[..8] {
                unsafe { (at as *mut u32).write_unaligned(0) };
            }
            CALLED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let ok = if rust {
                unsafe {
                    start_race_sounds(&before, &mut |p, r| {
                        handler(p, r);
                        Ok(())
                    })
                }
                .is_ok()
            } else {
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            (
                ok,
                std::mem::take(&mut *CALLED.lock().unwrap()),
                capture(&regions),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("count {count:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
