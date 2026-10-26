//! `net::shut_down` (0x4184D6): the network at the game's end: `net::stop`, then the
//! received messages' mutex (`dword_52AB6C`) and the receiving thread's event
//! (`dword_52AB68`) taken out (`xchg` with 0) and closed (`sub_411092`, `sub_41104F`:
//! `CloseHandle` if not 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. Two callers: `start`'s way out (0x4010EE) and `sub_4104F2`. Its calls
//! through `part` (in the game `stop`'s Rust version and `CloseHandle`). A call in its first
//! five bytes: private copy (0x22 bytes, three calls relocated), manual patch. It may wait on
//! the receiving thread: the thunk does not wait for self-tests.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{CLOSE_EVENT, CLOSE_MUTEX, EVENT, MUTEX, Part, STOP, a, net_entry};
use std::sync::atomic::{AtomicU32, Ordering};

/// The Rust version on the handles at `mutex` and `event`.
pub fn shut_down_with(r: &mut Regs, mutex: u32, event: u32, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    part(STOP, &mut g)?;
    for (at, place) in [(mutex, CLOSE_MUTEX), (event, CLOSE_EVENT)] {
        g.eax = unsafe { &*(at as *const AtomicU32) }.swap(0, Ordering::SeqCst);
        part(place, &mut g)?;
    }
    Ok(())
}

/// The Rust version.
pub fn shut_down(r: &mut Regs, part: Part) -> Result<(), Refused> {
    shut_down_with(r, a(MUTEX), a(EVENT), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, shut_down)
}
