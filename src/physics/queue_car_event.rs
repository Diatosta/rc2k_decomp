//! `physics::queue_car_event` (0x4419EB): an entry added to the work car's queue of up
//! to 7 (`g_car` +0xF98 the count): eax = `g_sceneClock` + esi; unless the count is 7,
//! the time into +0xF58[count], ebx into +0xF78[count], the count up by one; then +0xF9C
//! 0 and +0xFA0 0 for the first entry, else +0xF9C less 0x20. A reading: timed events
//! of the car (under `physics::pace_notes_step`).
//!
//! In:  esi a delay, ebx the event
//! Out: eax the time, ecx the count before; the rest kept; no flags read (the caller
//!      `popa`s)
//!
//! Caller: `sub_44194D`. Refuses (before anything) a count above 7 (the original writes
//! past the queue).

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::cells::{get, get_list, put_changed, put_list_changed};
use crate::physics::tables_store::PHYSICS;
use rc2k_core::physics::car_events::CarEvents;

/// Where the variables are: the clock, the times, the events, the count, +0xF9C, +0xFA0.
#[derive(Clone, Copy)]
pub struct QueuePlaces {
    pub clock: u32,
    pub times: u32,
    pub events: u32,
    pub count: u32,
    pub f9c: u32,
    pub fa0: u32,
}

impl QueuePlaces {
    pub fn game() -> Self {
        Self {
            clock: (PHYSICS.scene_clock.addr()),
            times: (CAR.car_event_times[0].addr()),
            events: (CAR.car_event_codes[0].addr()),
            count: (CAR.car_event_count.addr()),
            f9c: (CAR.car_event_limit.addr()),
            fa0: (CAR.car_event_position.addr()),
        }
    }
}

impl QueuePlaces {
    /// The queue at these places.
    pub fn load(&self) -> CarEvents {
        CarEvents {
            times: get_list(self.times),
            events: get_list(self.events),
            count: get(self.count),
            level: get(self.f9c),
            shown: get(self.fa0),
        }
    }

    /// What changed from `was` to `now` stored back.
    pub fn store(&self, was: &CarEvents, now: &CarEvents) {
        put_list_changed(self.times, &was.times, &now.times);
        put_list_changed(self.events, &was.events, &now.events);
        put_changed(self.count, was.count, now.count);
        put_changed(self.f9c, was.level, now.level);
        put_changed(self.fa0, was.shown, now.shown);
    }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::car_events::CarEvents::queue`.
pub fn queue_car_event_at(p: &QueuePlaces, esi: u32, ebx: u32) -> Result<(), Refused> {
    let was = p.load();
    let mut q = was;
    q.queue(get(p.clock), esi, ebx)?;
    p.store(&was, &q);
    Ok(())
}
