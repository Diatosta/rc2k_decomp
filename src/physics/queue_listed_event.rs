//! For an event number below 0x20, the event `listed_events[n]`; unless it is -1,
//! `physics::queue_car_event` runs with it and no delay.
//!
//! Refuses what `queue_car_event` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::figures::LISTED_EVENTS;
use crate::physics::queue_car_event::queue_car_event;
use rc2k_core::physics::car_events::listed_event;

pub(super) fn queue_listed_event(car: &mut Car, scene_clock: u32, ebx: u32) -> Result<(), Refused> {
    let Some((event, queued)) = listed_event(ebx, |n| LISTED_EVENTS[n as usize]) else {
        return Ok(());
    };
    if queued {
        queue_car_event(car, scene_clock, 0, event)?;
    }
    Ok(())
}
