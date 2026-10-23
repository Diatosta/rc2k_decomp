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
//! It jumps back to its own start: hooked by a private copy.
//! Callers: `physics::pace_notes_step`, and itself.
//! Refuses (before anything) a count above 7.
//! Self-test: a copy of the original (the clock and the queue the test's) against the
//! core: counts 0 .. 7 with times behind and not, +0xFA0 about +0xF9C.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, ESI, Regs};
use crate::physics::cells::get;
use crate::physics::queue_car_event::QueuePlaces;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version on the variables at `p` (esi and edi after a shift: the events'
/// eighth slot and the one after).
// Converted: `rc2k_core::physics::car_events::CarEvents::expire`.
pub unsafe fn expire_car_events_at(p: &QueuePlaces) -> Result<Left, Refused> {
    unsafe {
        let was = p.load();
        let mut q = was;
        let e = q.expire(get(p.clock))?;
        p.store(&was, &q);
        let left = Left::nothing().with(EAX, e.shown).with(ECX, 4);
        Ok(if e.shifted {
            left.with(ESI, p.events + 0x20).with(EDI, p.events + 0x1C)
        } else {
            left
        })
    }
}

/// The Rust version.
pub unsafe fn expire_car_events() -> Result<Left, Refused> {
    unsafe { expire_car_events_at(&QueuePlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { expire_car_events() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
