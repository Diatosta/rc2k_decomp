//! `physics::queue_car_event` (0x4419EB): an entry added to the work car's queue of up
//! to 7 (`g_car` +0xF98 the count): eax = `g_sceneClock` + esi; unless the count is 7,
//! the time into +0xF58[count], ebx into +0xF78[count], the count up by one; then +0xF9C
//! 0 and +0xFA0 0 for the first entry, else +0xF9C less 0x20. A reading: timed events
//! of the car (under `physics::pace_notes_step`).
//!
//! Takes: a delay, the event
//! Out: eax the time, ecx the count before; the rest kept; no flags read (the caller
//!      `popa`s)
//!
//! Caller: `sub_44194D`. Refuses (before anything) a count above 7 (the original writes
//! past the queue).

use crate::errors::Refused;
use crate::physics::cells::{set_changed, set_values_changed, values};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use rc2k_core::physics::car_events::CarEvents;

/// The work car's queue.
pub fn load(race: &Race) -> CarEvents {
    CarEvents {
        times: values(&race.shared.car.car_event_times),
        events: values(&race.shared.car.car_event_codes),
        count: race.shared.car.car_event_count.get(),
        level: race.shared.car.car_event_limit.get(),
        shown: race.shared.car.car_event_position.get(),
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(shared: &Shared, was: &CarEvents, now: &CarEvents) {
    set_values_changed(&shared.car.car_event_times, &was.times, &now.times);
    set_values_changed(&shared.car.car_event_codes, &was.events, &now.events);
    set_changed(&shared.car.car_event_count, was.count, now.count);
    set_changed(&shared.car.car_event_limit, was.level, now.level);
    set_changed(&shared.car.car_event_position, was.shown, now.shown);
}

/// The Rust version.
pub fn queue_car_event(race: &Race, esi: u32, ebx: u32) -> Result<(), Refused> {
    let was = load(race);
    let mut q = was;
    q.queue(race.shared.physics_tables.scene_clock.get(), esi, ebx)?;
    store(&race.shared, &was, &q);
    Ok(())
}
