//! `physics::count_down_start` (0x436AFB): the start's count-down. Unless `g_car` +0xA8
//! (`unk_71A080`) is set, and once the timer +0x14 less 0x320 is not below 0 (by its
//! sign): the count +0x8C (`unk_71A064`), when not 0, goes down by one; at whole
//! hundreds of 8 or less (`div` by 100, ebx 100) `physics::set_flag_5165a0` (a reading:
//! the count-down's beeps).
//!
//! In:  nothing
//! Out: eax the timer less 0x320 when it got that far; after the `div` eax the hundreds,
//!      ebx 100, edx the rest; after the call eax as `set_flag_5165a0` leaves it; the
//!      rest kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_436790` (0x4367D4).
//! Refuses what `set_flag_5165a0` refuses.
//! Self-test: a copy of the original (its variables the test's, its call to the twin's
//! test copy on test cells) against the core: counts about the hundreds, both switches.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, EDX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::physics::listed_car_flags::{CURRENT_CAR, TABLE, set_flag_at, twin_copy};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::start::{CarStart, StartCount};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::count_down_start", rva(0x436AFB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Where the variables are: +0xA8, the timer, the count; the twin's three.
#[derive(Clone, Copy)]
pub struct CountPlaces {
    pub stop: u32,
    pub timer: u32,
    pub count: u32,
    pub current: u32,
    pub listed: u32,
    pub flag: u32,
}

impl CountPlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            stop: g(0x71A080),
            timer: g(0x719FEC),
            count: g(0x71A064),
            current: g(CURRENT_CAR),
            listed: g(TABLE),
            flag: g(0x5165A0),
        }
    }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::start::CarStart::count_down_start`.
pub unsafe fn count_down_start_at(p: &CountPlaces) -> Result<Left, Refused> {
    unsafe {
        let mut c = CarStart {
            finished: at(p.stop),
            timer: at(p.timer),
            countdown: at(p.count),
            ..CarStart::default()
        };
        match c.count_down_start() {
            StartCount::Stopped => Ok(Left::nothing()),
            StartCount::Waiting(eax) => Ok(Left::nothing().with(EAX, eax)),
            StartCount::Counted {
                hundreds,
                rest,
                beep,
            } => {
                (p.count as *mut u32).write_unaligned(c.countdown);
                let left = if beep {
                    set_flag_at(p.current, p.listed, p.flag)?
                } else {
                    Left::nothing().with(EAX, hundreds)
                };
                Ok(left.with(EBX, 100).with(EDX, rest))
            }
        }
    }
}

/// The Rust version.
pub unsafe fn count_down_start() -> Result<Left, Refused> {
    unsafe { count_down_start_at(&CountPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { count_down_start() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let p = CountPlaces::game();
    dispatch_state(&HOOK, r, call_original, &[(p.count, 4), (p.flag, 4)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // stop, timer, count, current, flag, 8 listed.
    let mut cells = vec![0u32; 5 + 8];
    let base = cells.as_mut_ptr() as u32;
    let p = CountPlaces {
        stop: base,
        timer: base + 4,
        count: base + 8,
        current: base + 12,
        flag: base + 16,
        listed: base + 20,
    };
    let inner = match twin_copy(
        &crate::physics::set_flag_5165a0::HOOK,
        0x45D16D,
        0x5165A0,
        p.current,
        p.listed,
        p.flag,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x436AFB,
            len: 0x42,
            entry: &[0x83, 0x3D, 0x80, 0xA0, 0x71, 0x00, 0x00],
            calls: &[0x3C],
            jumps_out: &[],
        },
        &|r| (r == rva(0x45D16D)).then_some(target),
        &[(0x71A080, p.stop), (0x719FEC, p.timer), (0x71A064, p.count)],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for (stop, timer) in [(0u32, 0x400u32), (1, 0x400), (0, 0x31F)] {
        for count in [0u32, 1, 101, 100, 801, 901, 900, 250] {
            let fill = || unsafe {
                for k in 0..13u32 {
                    ((base + 4 * k) as *mut u32).write(0x5A5A_0000 | k);
                }
                (p.stop as *mut u32).write(stop);
                (p.timer as *mut u32).write(timer);
                (p.count as *mut u32).write(count);
                (p.current as *mut u32).write(count % 8);
                ((p.listed + 4 * (count % 8)) as *mut u32).write(if count % 3 == 1 {
                    u32::MAX
                } else {
                    7
                });
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { count_down_start_at(&p) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("stop {stop}, timer {timer:#x}, count {count}: regs_ok={regs_ok}; the memory differs")
            });
        }
    }
    drop((copy, inner, cells));
    t.finish()
}
