//! Ages the work car's event queue (`physics::queue_car_event`'s): while the count is not 0 and the
//! first entry's time is 3000 or more behind the scene clock (`scene_clock`), the two lists shift
//! down by one (seven dwords each; each list has an eighth slot, which the shift reads), the count
//! goes down, and the two write counters go up by 0x20 and 0x40. Then the second counter moves 4
//! towards the first (not past it; left when at or below it). It loops to its own start.
//!
//! Refuses (before anything) a count above 7.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::queue_car_event::{load, store};
use crate::physics::tables_store::PhysicsTables;

pub(super) fn expire_car_events(car: &Car, tables: &PhysicsTables) -> Result<(), Refused> {
    let was = load(car);
    let mut q = was;
    q.expire(tables.scene_clock.get())?;
    store(car, &was, &q);
    Ok(())
}
