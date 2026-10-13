//! `sound::step_race_clock` (0x40F20E): says whether a turn of the race's sounds is due.
//!
//! In:  nothing
//! Out: carry set = not yet; every register preserved
//! With fewer than 20 ms since the last turn (unsigned): carry set, nothing stored.
//! Else the last turn's time moves on by what has passed; that, held to 1000, is kept
//! as the turn's milliseconds, 1000 / it as the turns a second (a float), the turns
//! counted go up by one, and the carry is clear (`sound/race_clock.rs`). The zero flag
//! is clear either way.
//! One caller, `sub_40D5F3` (the race's sounds, every frame), which does `jb`.
//! Shadow mode runs the original alone (the clock moves) and checks what it left
//! against the Rust version with a clock that fits.
//! Self-test: a redirected copy with the test's clock.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{capture, restore};
use crate::sound::race_clock::{regions, step_race_clock};
use crate::system::tick_count::tick_count;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::step_race_clock", rva(0x40F20E), thunk, &ORIGINAL).self_test(self_test);

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
        let due = unsafe { step_race_clock(tick_count()) };
        r.set_carry(!due);
        r.set_zero(false);
        return;
    }
    let regions = regions();
    let (before, earliest) = (capture(&regions), tick_count());
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let (theirs, latest) = (capture(&regions), tick_count());
    let stepped = !o.carry();
    // The clock the original read: its new "last turn" if it stepped; else any clock
    // between the two readings here must leave things alone as well.
    let now = if stepped {
        u32::from_le_bytes(theirs[..4].try_into().unwrap())
    } else {
        earliest
    };
    restore(&regions, &before);
    let mine_stepped = unsafe { step_race_clock(now) };
    let mine = capture(&regions);
    restore(&regions, &theirs);
    let in_time = now.wrapping_sub(earliest) <= latest.wrapping_sub(earliest);
    if mine != theirs || mine_stepped != stepped || !in_time || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!(
                "clock {earliest:#x} .. {latest:#x}, before {before:x?}: original stepped {stepped} left {theirs:x?}; rust stepped {mine_stepped} left {mine:x?}"
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----

const LEN: usize = 0x49;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x1D, 0xE8, 0x24, 0x51, 0x00, 0xE8];
/// `system::tick_count`.
const CALLS: [usize; 1] = [0x07];

static CLOCK: AtomicU32 = AtomicU32::new(0);

fn handler(_: u32, r: &mut Regs) {
    r.eax = CLOCK.load(Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x40F20E)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    // (the last turn's time, the clock)
    let mut times = vec![(0xFFFF_FFF0u32, 0x10u32), (0x10, 0xFFFF_FFF0), (5000, 4999)];
    for passed in [
        0u32, 1, 0x13, 0x14, 0x15, 33, 50, 333, 999, 1000, 1001, 7000,
    ] {
        times.push((1000, 1000 + passed));
    }
    for (last, clock) in times {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            let mut start = vec![0xAA; saved.len()];
            start[..4].copy_from_slice(&last.to_le_bytes());
            restore(&regions, &start);
            CLOCK.store(clock, Ordering::SeqCst);
            set_handler(handler);
            let (preserved, due) = if rust {
                (true, step_race_clock(clock))
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                (
                    ro.preserved_except(&before, 0) && ro.eflags & 0x40 == 0,
                    !ro.carry(),
                )
            };
            (preserved, due, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("last turn {last:#x}, clock {clock:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
