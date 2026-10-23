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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, Regs, game_addr, rva};
use crate::physics::listed_car_flags::{CURRENT_CAR, TABLE, set_flag_at};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::start::{CarStart, StartCount};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
