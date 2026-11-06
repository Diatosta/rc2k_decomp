//! `physics::false_start` (0x43B9B4): the work car's start held back (`g_car` +0x14 ..
//! +0x1C, a reading: a jump start's penalty). +0x18 (`dword_719FF0`) is set once the car
//! is in gear; while it is set and +0x1C (`dword_719FF4`) is 0, a timer +0x14
//! (`dword_719FEC`, in eax) below 0x320 (by the sign of the difference, `jns`) starts the
//! count +0x1C at 500, with +0x71C (`dword_71A6F4`) 0xCC, +0x730 0 and +0x72C -1 (the
//! notice block, by its look). Each call the count goes down; on reaching 0: +0x88
//! (`dword_71A060`) + 1000, +0x14 and +0x18 0, +0x730 0, +0x72C 300, +0x724 0, +0x71C
//! 0xCA, +0x720 0, +0x728 0, and `race::start_car` (the car put back at its start).
//!
//! In:  nothing
//! Out: eax the timer when it was compared; when `start_car` ran, the registers as it
//!      leaves them; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43B635` (0x43B9A9), every physics step. Refuses what `start_car` refuses.

use crate::errors::Refused;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use rc2k_core::physics::notices::Notice;
use rc2k_core::physics::start::CarStart;

/// The Rust version, then `race::start_car` (it takes no register: called with none). The
/// registers it leaves (eax the timer when set) are read by none of its callers.
// Converted: `rc2k_core::physics::start::CarStart::false_start`.
pub fn false_start(shared: &Shared) -> Result<(), Refused> {
    let c = &shared.car;
    let was = CarStart {
        timer: c.run_ticks.get(),
        held: c.start_in_gear.get(),
        count: c.false_start_count.get(),
        time: c.stage_time.get(),
        ..CarStart::default()
    };
    let notice_was = Notice {
        id: c.notice_id.get(),
        w6f8: c.notice_counter.get(),
        last: c.notice_last.get(),
        shown: c.notice_shown.get(),
        time: c.notice_time.get(),
        w708: c.notice_time_b.get(),
    };
    let (mut car, mut notice) = (was, notice_was);
    let done = car.false_start(c.gear.get(), &mut notice);
    let changes: [(&Dword, u32, u32); 10] = [
        (&c.start_in_gear, was.held, car.held),
        (&c.false_start_count, was.count, car.count),
        (&c.stage_time, was.time, car.time),
        (&c.run_ticks, was.timer, car.timer),
        (&c.notice_id, notice_was.id, notice.id),
        (&c.notice_counter, notice_was.w6f8, notice.w6f8),
        (&c.notice_last, notice_was.last, notice.last),
        (&c.notice_shown, notice_was.shown, notice.shown),
        (&c.notice_time, notice_was.time, notice.time),
        (&c.notice_time_b, notice_was.w708, notice.w708),
    ];
    for (at, before, now) in changes {
        if now != before {
            at.set(now);
        }
    }
    if done.put_back {
        { crate::race::car_starts::start_car() }?;
    }
    Ok(())
}
