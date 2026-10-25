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
//! Caller: `sub_436790` (0x4367D4). Refuses what `set_flag_5165a0` refuses.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::listed_car_flags::set_flag_at;
use crate::race::frame_store::FRAME_WORK;
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
        Self {
            stop: CAR.finish_flag.addr() as u32,
            timer: (CAR.run_ticks.addr() as u32),
            count: (CAR.time_left.addr() as u32),
            current: crate::race::work_store::RACE_WORK.current_car.addr() as u32,
            listed: FRAME_WORK.listed_car_sounds[0].addr() as u32,
            flag: crate::sound::store::STAGE_SOUNDS.once[1].addr() as u32,
        }
    }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::start::CarStart::count_down_start`.
pub unsafe fn count_down_start_at(p: &CountPlaces) -> Result<(), Refused> {
    unsafe {
        let mut c = CarStart {
            finished: at(p.stop),
            timer: at(p.timer),
            countdown: at(p.count),
            ..CarStart::default()
        };
        match c.count_down_start() {
            StartCount::Stopped | StartCount::Waiting(_) => Ok(()),
            StartCount::Counted { beep, .. } => {
                (p.count as *mut u32).write_unaligned(c.countdown);
                if beep {
                    set_flag_at(p.current, p.listed, p.flag)?;
                }
                Ok(())
            }
        }
    }
}

/// The Rust version.
pub unsafe fn count_down_start() -> Result<(), Refused> {
    unsafe { count_down_start_at(&CountPlaces::game()) }
}
