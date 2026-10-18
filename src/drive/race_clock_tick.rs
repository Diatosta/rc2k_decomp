//! `drive::race_clock_tick` (0x445554): the race's clock in tenths. `scene::restart_clock`
//! (the tick count into `g_sceneClock`), into `dword_61383C`; the last tick kept
//! (`dword_613840`) brought to within 1000 of it (by the sign of the `cmp`); the tenths
//! since (`div` by 10) into `dword_613844` and added back (x 10) to `dword_613840`.
//!
//! In:  nothing
//! Out: eax the tenths x 10, ebx 10, edx the product's high dword (0); the rest kept; no
//!      flags read
//!
//! Caller: `drive::race_step` (`sub_4454D2`). It begins with a call: hooked by a private
//! copy of its 0x55 bytes (the call relocated) and a hand-made patch.
//! Self-test: a copy whose clock call goes to a stand-in giving the test's times, against
//! the Rust version with the same times: within 1000 and beyond, round tenths and not.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::physics::flow::{a, g, p};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::race_clock_tick", rva(0x445554), thunk, &ORIGINAL)
    .private_copy(0x55)
    .relocate_calls(&[0x0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn tick_from(now: u32) -> Left {
    p(0x61383C, now);
    if (g(0x61383C).wrapping_sub(g(0x613840)).wrapping_sub(0x3E8) as i32) >= 0 {
        p(0x613840, g(0x61383C).wrapping_sub(0x3E8));
    }
    let since = g(0x61383C).wrapping_sub(g(0x613840));
    let tenths = since / 10;
    p(0x613844, tenths);
    let product = u64::from(tenths) * 10;
    p(0x613840, g(0x613840).wrapping_add(product as u32));
    Left::nothing()
        .with(EAX, product as u32)
        .with(EBX, 10)
        .with(EDX, (product >> 32) as u32)
}

/// The Rust version.
pub unsafe fn race_clock_tick() -> Left {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    tick_from(now)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { race_clock_tick() })
}

extern "C" fn dispatch(r: &mut Regs) {
    // The clock moves between two runs: the original alone in shadow mode.
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    rust(r).expect("infallible").apply(r);
}

static NOW: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    let now = NOW.load(Ordering::SeqCst);
    p(0x613830, now);
    r.eax = now;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            crate::framework::game_addr(rva(0x445554)),
            0x55,
            &[0xE8, 0x28, 0xAE, 0x00, 0x00],
            &[0x00],
            &|_| Some(stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let all = [(a(0x613830), 4), (a(0x61383C), 12)];
    let saved = capture(&all);
    for (now, last) in [
        (5000u32, 4990u32),
        (5000, 3000),
        (5007, 5000),
        (12, 0xFFFF_FF00),
        (999, 0),
    ] {
        NOW.store(now, Ordering::SeqCst);
        p(0x613840, last);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (ro, capture(&all));
        p(0x613840, last);
        p(0x613830, now);
        let mine = tick_from(now);
        let state = capture(&all);
        t.check(mine.matches(&Regs::SENTINEL, &theirs.0) && state == theirs.1, || {
            let o = &theirs.0;
            format!(
                "now {now}, last {last}: original eax {:x} ebx {:x} edx {:x} {:x?}; rust {mine:?} {state:x?}",
                o.eax, o.ebx, o.edx, theirs.1
            )
        });
    }
    restore(&all, &saved);
    t.finish()
}
