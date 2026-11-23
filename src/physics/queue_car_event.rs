//! Adds an entry to the work car's queue of up to 7 (`car_event_count` the count): the time is
//! `scene_clock` plus the delay; unless the count is 7, the time goes into
//! `car_event_times[count]`, the event into `car_event_codes[count]` and the count goes up by one;
//! then `car_event_limit` and `car_event_position` are 0 for the first entry, else
//! `car_event_limit` is reduced by 0x20. The events are the pace-note symbols the HUD stacks
//! (`race::panel_marks`), each shown for 3000 of the scene clock.
//!
//! Refuses (before anything) a count above 7.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::cells::{set_changed, set_values_changed, values};
use rc2k_core::physics::car_events::CarEvents;

/// The work car's queue.
pub fn load(car: &mut Car) -> CarEvents {
    CarEvents {
        times: values(&car.car_event_times),
        events: values(&car.car_event_codes),
        count: car.car_event_count,
        level: car.car_event_limit,
        shown: car.car_event_position,
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(car: &mut Car, was: &CarEvents, now: &CarEvents) {
    set_values_changed(&mut car.car_event_times, &was.times, &now.times);
    set_values_changed(&mut car.car_event_codes, &was.events, &now.events);
    set_changed(&mut car.car_event_count, was.count, now.count);
    set_changed(&mut car.car_event_limit, was.level, now.level);
    set_changed(&mut car.car_event_position, was.shown, now.shown);
}

pub(super) fn queue_car_event(
    car: &mut Car,
    scene_clock: u32,
    esi: u32,
    ebx: u32,
) -> Result<(), Refused> {
    let was = load(car);
    let mut q = was;
    q.queue(scene_clock, esi, ebx)?;
    store(car, &was, &q);
    Ok(())
}
