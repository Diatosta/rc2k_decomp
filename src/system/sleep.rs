//! `system::sleep` (0x41100C): `Sleep(eax)`.
//!
//! In:  eax = milliseconds
//! Out: every register preserved
//! 34 callers. Shadow mode runs the original alone: sleeping twice would change the game's
//! timing. The thunk is not gated, so a thread asleep in here does not hold up a self-test.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;
use windows_sys::Win32::System::Threading::Sleep;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("system::sleep", rva(0x41100C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version ----

pub fn sleep(milliseconds: u32) {
    unsafe { Sleep(milliseconds) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    sleep(r.eax);
}

// ---- self-test: both sleep at least as long as asked, and not wildly longer ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for milliseconds in [0u32, 1, 30] {
        let before = Regs {
            eax: milliseconds,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        let start = Instant::now();
        unsafe { call_original(&mut ro) };
        let theirs = start.elapsed().as_millis() as u32;

        let start = Instant::now();
        sleep(milliseconds);
        let mine = start.elapsed().as_millis() as u32;

        // Timers are coarse: a sleep may end a little early and a good deal late.
        let plausible = |took: u32| took + 2 >= milliseconds && took <= milliseconds + 200;
        t.check(
            plausible(theirs) && plausible(mine) && ro.preserved_except(&before, 0),
            || format!("{milliseconds} ms: original took {theirs} ms, rust {mine} ms"),
        );
    }
    t.finish()
}
