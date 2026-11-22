//! Holds the work car's start back (a reading: a jump start's penalty). `start_in_gear` is set
//! once the car is in gear; while it is set and `false_start_count` is 0, `run_ticks` below
//! 0x320 starts the count at 500, with the notice block (`notice_id` 0xCC, `notice_time_b` 0,
//! `notice_time` -1). Each call the count goes down; on reaching 0: `stage_time` + 1000,
//! `run_ticks` and `start_in_gear` 0, `notice_time_b` 0, `notice_time` 300, `notice_last` 0,
//! `notice_id` 0xCA, `notice_counter` 0, `notice_shown` 0, and `race::start_car` (the car is
//! put back at its start).
//!
//! Refuses what `start_car` refuses.

use crate::errors::Refused;
use crate::physics::step_families::ResetInputs;
use rc2k_core::physics::notices::Notice;
use rc2k_core::physics::start::CarStart;

/// The false start check, then `race::start_car`.
// Core: `rc2k_core::physics::start::CarStart::false_start`.
pub fn false_start(
    inputs: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let mut car = inputs.shared.car_mut();
    let c = &*car;
    let was = CarStart {
        timer: c.run_ticks,
        held: c.start_in_gear,
        count: c.false_start_count,
        time: c.stage_time,
        ..CarStart::default()
    };
    let notice_was = Notice {
        id: c.notice_id,
        counter: c.notice_counter,
        last: c.notice_last,
        shown: c.notice_shown,
        time: c.notice_time,
        second_time: c.notice_time_b,
    };
    let (mut start, mut notice) = (was, notice_was);
    let done = start.false_start(car.gear, &mut notice);
    car.start_in_gear = start.held;
    car.false_start_count = start.count;
    car.stage_time = start.time;
    car.run_ticks = start.timer;
    car.notice_id = notice.id;
    car.notice_counter = notice.counter;
    car.notice_last = notice.last;
    car.notice_shown = notice.shown;
    car.notice_time = notice.time;
    car.notice_time_b = notice.second_time;
    drop(car);
    if done.put_back {
        { crate::race::car_starts::start_car(inputs, scene, net_race) }?;
    }
    Ok(())
}
