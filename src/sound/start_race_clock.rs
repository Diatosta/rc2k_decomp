//! `sound::start_race_clock` (0x40F1E9): sets the clock of the race's sounds going.
//!
//! In:  nothing
//! Out: every register preserved
//! The last turn's time = the clock now, the turns a second = 50.0, the turns counted =
//! 0 (`sound/race_clock.rs`). Two callers, `sub_40D378` and `sub_40D592`.
//! Shadow mode runs the original alone (the clock moves) and checks what it left.
//! Manual patch with a private copy: its second instruction is a call.
//! Self-test: a redirected copy with the test's clock.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{capture, restore};
use crate::sound::race_clock::{regions, start_race_clock};
use crate::system::tick_count::tick_count;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::start_race_clock", rva(0x40F1E9), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        unsafe { start_race_clock(tick_count()) };
        return;
    }
    let regions = regions();
    let (before, earliest) = (capture(&regions), tick_count());
    unsafe { call_original(r) };
    let (theirs, latest) = (capture(&regions), tick_count());
    // The Rust version with the clock the original read, which is its first store.
    let now = u32::from_le_bytes(theirs[..4].try_into().unwrap());
    restore(&regions, &before);
    unsafe { start_race_clock(now) };
    let mine = capture(&regions);
    restore(&regions, &theirs);
    if mine != theirs || now.wrapping_sub(earliest) > latest.wrapping_sub(earliest) {
        HOOK.mismatch(|| {
            format!("clock {earliest:#x} .. {latest:#x}: original left {theirs:x?}; rust {mine:x?}")
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x25;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x35, 0x1E, 0x00, 0x00, 0xA3, 0xE8];
/// `system::tick_count`.
const CALLS: [usize; 1] = [0x01];

static CLOCK: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    r.eax = CLOCK.load(Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x40F1E9)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    for clock in [0u32, 1, 0x1234_5678, 0x7FFF_FFFF, 0xFFFF_FFFF] {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            CLOCK.store(clock, Ordering::SeqCst);
            set_handler(handler);
            let preserved = if rust {
                start_race_clock(clock);
                true
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (preserved, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("clock {clock:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
