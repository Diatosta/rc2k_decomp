//! `physics::false_start` (0x43B9B4): the work car's start held back (a reading: a jump start's
//! penalty). `start_in_gear` is set once the car is in gear; while it is set and `false_start_count`
//! is 0, `run_ticks` below 0x320 (by the sign of the difference) starts the count at 500, with the
//! notice block (`notice_id` 0xCC, `notice_time_b` 0, `notice_time` -1). Each call the count goes
//! down; on reaching 0: `stage_time` + 1000, `run_ticks` and `start_in_gear` 0, `notice_time_b` 0,
//! `notice_time` 300, `notice_last` 0, `notice_id` 0xCA, `notice_counter` 0, `notice_shown` 0, and
//! `race::start_car` (the car put back at its start).
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses what `start_car` refuses.

use crate::errors::Refused;
use crate::physics::step_inputs::StepInputs;
use crate::session::Dword;
use rc2k_core::physics::notices::Notice;
use rc2k_core::physics::start::CarStart;

/// The false start check, then `race::start_car`.
// Converted: `rc2k_core::physics::start::CarStart::false_start`.
pub fn false_start(inputs: &StepInputs<'_>) -> Result<(), Refused> {
    let c = inputs.car;
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
        { crate::race::car_starts::start_car(inputs) }?;
    }
    Ok(())
}
