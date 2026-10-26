//! `net::settings_done` (0x416F0B): the connection saved checked: the connections listed
//! (`net::find_connections_regs`) and the saved one among them
//! (`net::find_saved_connection`) kept again (`net::keep_connection`, its name fresh);
//! if either fails the saved connection forgotten (`unk_52AC94` 16 bytes, `unk_52ACA4` 64,
//! `system::zero_memory`); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::settings_loop` (on entering it). Its calls through `part`. It waits for the
//! receiving thread: ungated. Shadow mode runs the original alone. Self-test: a copy whose
//! calls go to stand-ins, against the Rust version: found, not found, none listed.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, SAVED_CONNECTION, SAVED_NAME, STOP, ZERO_MEMORY, a, lobby_entry};

const FIND_CONNECTIONS: u32 = 0x4174BD; // net::find_connections_regs
const FIND_SAVED: u32 = 0x4185DF; // net::find_saved_connection
const KEEP_CONNECTION: u32 = 0x417661; // net::keep_connection

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_done(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(FIND_CONNECTIONS, &mut r)?;
    let mut found = !r.carry();
    if found {
        part(FIND_SAVED, &mut r)?;
        found = !r.carry();
        if found {
            part(KEEP_CONNECTION, &mut r)?;
        }
    }
    if !found {
        for (at, len) in [(SAVED_CONNECTION, 0x10), (SAVED_NAME, 0x40)] {
            let mut c = Regs {
                ebx: a(at),
                ecx: len,
                ..r
            };
            part(ZERO_MEMORY, &mut c)?;
            r.eflags = c.eflags;
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
    lobby_entry(r, settings_done)
}
