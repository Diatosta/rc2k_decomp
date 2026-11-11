//! `physics::expire_car_events` (0x44196B): the work car's event queue
//! (`physics::queue_car_event`'s) aged: while the count (+0xF98) is not 0 and the first
//! entry's time is 3000 or more behind `g_sceneClock` (by the sign), the two lists shift
//! down by one (seven dwords from +4 each, `rep movsd`: each list has an eighth slot,
//! +0xF4C and +0xF6C, which the shift reads), the
//! count goes down, +0xF9C up 0x20 and +0xFA0 up 0x40; it loops by a `jmp` to its own
//! start. Then +0xFA0 moves 4 towards +0xF9C (not past it; left when at or below it, by
//! the sign).
//!
//! Callers: `physics::pace_notes_step`, and itself. Refuses (before anything) a count above
//! 7.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::queue_car_event::{load, store};
use crate::physics::tables_store::PhysicsTables;

pub fn expire_car_events(car: &Car, tables: &PhysicsTables) -> Result<(), Refused> {
    let was = load(car);
    let mut q = was;
    q.expire(tables.scene_clock.get())?;
    store(car, &was, &q);
    Ok(())
}
