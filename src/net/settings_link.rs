//! `net::settings_link` (0x416EEC): Enter on the settings screen's connection: the
//! connections listed (`net::find_connections_regs`), one chosen (`net::choose_connection`)
//! and kept (`net::keep_connection`); then the network stopped again (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::settings_loop`. Its calls through `part`. Long-running (a list screen):
//! ungated. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version: no connections, none chosen (carry, a negative
//! choice), one chosen.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, STOP, lobby_entry};

const FIND_CONNECTIONS: u32 = 0x4174BD; // net::find_connections_regs
const CHOOSE_CONNECTION: u32 = 0x4174C5; // net::choose_connection
const KEEP_CONNECTION: u32 = 0x417661; // net::keep_connection

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_link(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(FIND_CONNECTIONS, &mut r)?;
    if !r.carry() {
        part(CHOOSE_CONNECTION, &mut r)?;
        if !r.carry() && (r.eax as i32) >= 0 {
            part(KEEP_CONNECTION, &mut r)?;
        }
    }
    part(STOP, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, settings_link)
}
