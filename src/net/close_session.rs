//! `net::close_session` (0x418DD3): with a session open (`dword_52ACFC` 1), the switch made
//! 0 and DirectPlay's `Close` (`IDirectPlay4A` +0x10) called through `g_dplay`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp dword_52ACFC, 1` when it
//!      is not 1, else those the method left (not modelled: the Rust version leaves the
//!      compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `Close` from Rust (`net::dplay`, section 15). Refuses
//! nothing. Shadow mode runs the original alone. Self-test: a copy whose two variables are
//! the test's and whose DirectPlay is a test object noting its arguments, against the Rust
//! version: open, not open, 2.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{CLOSE, method};
use crate::net::lobby::cmp_flags;
use crate::net::message_vars::{MessageVars, get, set};
use crate::net::{Part, net_entry};

/// The Rust version on the variables at `v`.
pub fn close_session_with(r: &mut Regs, v: &MessageVars) {
    let open = get(v.session_open);
    cmp_flags(r, open, 1);
    if open == 1 {
        set(v.session_open, 0);
        unsafe { method(get(v.dplay), CLOSE, &[]) };
    }
}

/// The Rust version.
pub fn close_session(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    close_session_with(r, &MessageVars::game());
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, close_session)
}
