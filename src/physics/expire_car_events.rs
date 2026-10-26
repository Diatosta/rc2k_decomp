//! `physics::expire_car_events` (0x44196B): the work car's event queue
//! (`physics::queue_car_event`'s) aged: while the count (+0xF98) is not 0 and the first
//! entry's time is 3000 or more behind `g_sceneClock` (by the sign), the two lists shift
//! down by one (seven dwords from +4 each, `rep movsd`: each list has an eighth slot,
//! +0xF4C and +0xF6C, which the shift reads), the
//! count goes down, +0xF9C up 0x20 and +0xFA0 up 0x40; it loops by a `jmp` to its own
//! start. Then +0xFA0 moves 4 towards +0xF9C (not past it; left when at or below it, by
//! the sign).
//!
//! In:  nothing
//! Out: eax +0xFA0, ecx 4; after a shift esi 0x71AF70, edi 0x71AF6C; the rest kept; no
//!      flags read (the callers return)
//!
//! Callers: `physics::pace_notes_step`, and itself. Refuses (before anything) a count above
//! 7.

use crate::errors::Refused;

use crate::physics::cells::get;
use crate::physics::queue_car_event::QueuePlaces;

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::car_events::CarEvents::expire`.
pub fn expire_car_events_at(p: &QueuePlaces) -> Result<(), Refused> {
    let was = p.load();
    let mut q = was;
    q.expire(get(p.clock))?;
    p.store(&was, &q);
    Ok(())
}
