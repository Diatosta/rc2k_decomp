//! For an event number below 0x20, the event `listed_events[n]`; unless it is -1,
//! `physics::queue_car_event` runs with it and no delay.
//!
//! Caller: `physics::queue_pace_note`. Refuses what `queue_car_event` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::queue_car_event::queue_car_event;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::car_events::listed_event;

pub fn queue_listed_event(car: &Car, tables: &PhysicsTables, ebx: u32) -> Result<(), Refused> {
    let Some((event, queued)) = listed_event(ebx, |n| tables.listed_events[n as usize].get())
    else {
        return Ok(());
    };
    if queued {
        queue_car_event(car, tables, 0, event)?;
    }
    Ok(())
}
