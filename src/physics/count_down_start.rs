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
use crate::physics::listed_car_flags::set_flag;
use rc2k_core::physics::start::{CarStart, StartCount};

/// The Rust version.
pub fn count_down_start() -> Result<(), Refused> {
    let mut c = CarStart {
        finished: CAR.finish_flag.get(),
        timer: CAR.run_ticks.get(),
        countdown: CAR.time_left.get(),
        ..CarStart::default()
    };
    match c.count_down_start() {
        StartCount::Stopped | StartCount::Waiting(_) => Ok(()),
        StartCount::Counted { beep, .. } => {
            CAR.time_left.set(c.countdown);
            if beep {
                set_flag(&crate::sound::owner::sound().race.stage.once[1])?;
            }
            Ok(())
        }
    }
}
