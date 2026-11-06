//! `physics::queue_listed_event` (0x44194D): for ebx below 0x20, the event
//! `dword_614C7C[ebx]` (into ebx); unless it is -1, `physics::queue_car_event` with no
//! delay (esi 0), inside `pusha` / `popa`.
//!
//! In:  ebx the event's number
//! Out: ebx the table's entry (when below 0x20); the rest kept; no flags read (the
//!      caller's next is an `inc`)
//!
//! Caller: `sub_4418B3`. Refuses what `queue_car_event` refuses.

use crate::errors::Refused;
use crate::physics::queue_car_event::queue_car_event;
use crate::race::owner::race;
use rc2k_core::physics::car_events::listed_event;

/// The Rust version.
pub fn queue_listed_event(ebx: u32) -> Result<(), Refused> {
    let Some((event, queued)) = listed_event(ebx, |n| {
        race().shared.physics_tables.listed_events[n as usize].get()
    }) else {
        return Ok(());
    };
    if queued {
        queue_car_event(0, event)?;
    }
    Ok(())
}
