//! `net::destroy_player` (0x41916B): DirectPlay's `DestroyPlayer` (`IDirectPlay4A` +0x24)
//! on `g_netPlayerId` through `g_dplay`, and the id made 0 — but only when it is 0 already
//! (`cmp g_netPlayerId, 0; jnz` past the call: the game's slip, section 16; this machine's
//! player is never destroyed, `DestroyPlayer(0)` is called instead).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp g_netPlayerId, 0` when
//!      it is not 0, else those the method left (not modelled: the Rust version leaves the
//!      compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `DestroyPlayer` from Rust (`net::dplay`, section 15).
//! Refuses nothing. Shadow mode runs the original alone. Self-test: a copy whose two
//! variables are the test's and whose DirectPlay is a test object noting its arguments,
//! against the Rust version: id 0, id not 0.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{DESTROY_PLAYER, method};
use crate::net::lobby::cmp_flags;
use crate::net::message_vars::{MessageVars, get, set};
use crate::net::{Part, net_entry};

/// The Rust version on the variables at `v`.
pub fn destroy_player_with(r: &mut Regs, v: &MessageVars) {
    let id = get(v.player_id);
    cmp_flags(r, id, 0);
    if id == 0 {
        unsafe { method(get(v.dplay), DESTROY_PLAYER, &[id]) };
        set(v.player_id, 0);
    }
}

/// The Rust version.
pub fn destroy_player(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    destroy_player_with(r, &MessageVars::game());
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, destroy_player)
}
