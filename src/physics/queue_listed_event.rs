//! `physics::queue_listed_event` (0x44194D): for ebx below 0x20, the event
//! `dword_614C7C[ebx]` (into ebx); unless it is -1, `physics::queue_car_event` with no
//! delay (esi 0), inside `pusha` / `popa`.
//!
//! In:  ebx the event's number
//! Out: ebx the table's entry (when below 0x20); the rest kept; no flags read (the
//!      caller's next is an `inc`)
//!
//! Caller: `sub_4418B3`.
//! Refuses what `queue_car_event` refuses.
//! Self-test: a copy of the original (the table the test's, its call to
//! `queue_car_event`'s test copy on test cells) against the core: numbers in and past the
//! table, entries -1 and not, queue counts 0 .. 7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, Regs, game_addr, rva};
use crate::physics::queue_car_event::{QueuePlaces, queue_car_event_at};
use crate::screens::harness::Left;
use rc2k_core::physics::car_events::listed_event;

const TABLE_RVA: usize = rva(0x614C7C); // dword_614C7C, 0x20 entries

/// The core: the table at `table`, the queue at `q`.
// Converted: `rc2k_core::physics::car_events::listed_event`.
pub unsafe fn queue_listed_event_at(
    table: u32,
    q: &QueuePlaces,
    ebx: u32,
) -> Result<Left, Refused> {
    let entry = |n: u32| unsafe { ((table + 4 * n) as *const u32).read_unaligned() };
    let Some((event, queued)) = listed_event(ebx, entry) else {
        return Ok(Left::nothing());
    };
    if queued {
        unsafe { queue_car_event_at(q, 0, event) }?;
    }
    Ok(Left::nothing().with(EBX, event))
}

/// The Rust version.
pub unsafe fn queue_listed_event(ebx: u32) -> Result<Left, Refused> {
    unsafe { queue_listed_event_at(game_addr(TABLE_RVA) as u32, &QueuePlaces::game(), ebx) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { queue_listed_event(r.ebx) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
