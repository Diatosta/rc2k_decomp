//! `physics::queue_car_event` (0x4419EB): an entry added to the work car's queue of up to 7
//! (`car_event_count` the count): the time is `g_sceneClock` plus the delay; unless the count is 7,
//! the time goes into `car_event_times[count]`, the event into `car_event_codes[count]`, the count
//! goes up by one; then `car_event_limit` and `car_event_position` are 0 for the first entry,
//! else `car_event_limit` is reduced by 0x20. The events are the pace-note symbols the HUD stacks
//! (`race::panel_marks`), each shown for 3000 of the scene clock.
//!
//! Takes: a delay, the event
//!
//! Caller: `physics::queue_listed_event`. Refuses (before anything) a count above 7 (the original
//! writes past the queue).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::cells::{set_changed, set_values_changed, values};
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::car_events::CarEvents;

/// The work car's queue.
pub fn load(car: &Car) -> CarEvents {
    CarEvents {
        times: values(&car.car_event_times),
        events: values(&car.car_event_codes),
        count: car.car_event_count.get(),
        level: car.car_event_limit.get(),
        shown: car.car_event_position.get(),
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(car: &Car, was: &CarEvents, now: &CarEvents) {
    set_values_changed(&car.car_event_times, &was.times, &now.times);
    set_values_changed(&car.car_event_codes, &was.events, &now.events);
    set_changed(&car.car_event_count, was.count, now.count);
    set_changed(&car.car_event_limit, was.level, now.level);
    set_changed(&car.car_event_position, was.shown, now.shown);
}

pub fn queue_car_event(
    car: &Car,
    tables: &PhysicsTables,
    esi: u32,
    ebx: u32,
) -> Result<(), Refused> {
    let was = load(car);
    let mut q = was;
    q.queue(tables.scene_clock.get(), esi, ebx)?;
    store(car, &was, &q);
    Ok(())
}
