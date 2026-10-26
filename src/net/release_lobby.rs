//! `net::release_lobby` (0x41900F): with a lobby object (`g_dplayLobby` not 0), the object
//! released (`IDirectPlayLobby2A` +8) and `g_dplayLobby` made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); without an object the flags of
//!      `cmp g_dplayLobby, 0`, else those `Release` left (not modelled: the Rust version
//!      leaves the compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`, section 15).
//! Refuses nothing. Shadow mode runs the original alone. Self-test: a copy whose variable
//! is the test's and whose lobby is a test object noting its arguments, against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::cmp_flags;
use crate::net::message_vars::{MessageVars, get, set};
use crate::net::{Part, net_entry};

/// The Rust version on the variables at `v`.
pub fn release_lobby_with(r: &mut Regs, v: &MessageVars) {
    let object = get(v.lobby);
    cmp_flags(r, object, 0);
    if object != 0 {
        unsafe { method(get(v.lobby), RELEASE, &[]) };
        set(v.lobby, 0);
    }
}

/// The Rust version.
pub fn release_lobby(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    release_lobby_with(r, &MessageVars::game());
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, release_lobby)
}
