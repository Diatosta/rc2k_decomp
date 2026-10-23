//! `net::lock_received` (0x418474): the received messages' mutex (`g_netMutex`) taken:
//! `system::wait_object` (0x4110A6: `WaitForSingleObject(eax, INFINITE)`, the flags of
//! `cmp result + 1, 1`) inside `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register kept; the flags `wait_object` left (zero: taken, carry: failed; no
//!      caller reads them)
//! Six callers (`net::copy_received`, `net::start_link`, `net::lobby_frame`,
//! `net::send_chat`, `net::chat_line_locked`, `net::take_messages`). The wait through
//! `part` (in the game `WaitForSingleObject` from Rust, `net::game_part`). Refuses nothing.
//! Shadow mode runs the original alone (the mutex). Self-test: a copy whose mutex variable
//! is the test's and whose wait goes to a noting stand-in (answering each flag setting),
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::message_vars::{MessageVars, get};
use crate::net::{Part, WAIT_OBJECT, net_entry};

/// The Rust version on the variables at `v`.
pub fn lock_received_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        eax: get(v.mutex),
        ..*r
    };
    part(WAIT_OBJECT, &mut c)?;
    r.eflags = c.eflags;
    Ok(())
}

/// The Rust version.
pub fn lock_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    lock_received_with(r, &MessageVars::game(), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, lock_received)
}
