//! `physics::queue_listed_event` (0x44194D): for ebx below 0x20, the event
//! `dword_614C7C[ebx]` (into ebx); unless it is -1, `physics::queue_car_event` with no
//! delay (esi 0).
//!
//! Takes: the event's number
//!
//! Caller: `physics::queue_pace_note`. Refuses what `queue_car_event` refuses.

use crate::errors::Refused;
use crate::physics::queue_car_event::queue_car_event;
use crate::race::owner::Race;
use rc2k_core::physics::car_events::listed_event;

pub fn queue_listed_event(race: &Race, ebx: u32) -> Result<(), Refused> {
    let Some((event, queued)) = listed_event(ebx, |n| {
        race.shared.physics_tables.listed_events[n as usize].get()
    }) else {
        return Ok(());
    };
    if queued {
        queue_car_event(race, 0, event)?;
    }
    Ok(())
}
