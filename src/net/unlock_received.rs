//! `net::unlock_received` (0x418481): the received messages' mutex (`g_netMutex`) given
//! back: `system::release_mutex` (0x4110B9: `ReleaseMutex(eax)`, the flags of
//! `cmp result, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept; the flags `release_mutex` left (zero: released; no caller
//!      reads them)
//! Six callers, those of `net::lock_received`. The release through `part` (in the game
//! `ReleaseMutex` from Rust, `net::game_part`). Refuses nothing. Shadow mode runs the
//! original alone. Self-test: as `net::lock_received`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::message_vars::{MessageVars, get};
use crate::net::{Part, RELEASE_MUTEX, net_entry};

/// The Rust version on the variables at `v`.
pub fn unlock_received_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        eax: get(v.mutex),
        ..*r
    };
    part(RELEASE_MUTEX, &mut c)?;
    r.eflags = c.eflags;
    Ok(())
}

/// The Rust version.
pub fn unlock_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    unlock_received_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, unlock_received)
}
