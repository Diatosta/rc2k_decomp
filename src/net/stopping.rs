//! `net::stopping` (0x417089): with the session on (`g_netSession` 1), the background
//! (`net::show_background`) and "stopping" shown (string 0x157, as `net::connecting` shows
//! its); then the network stopped (`net::stop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). Shadow
//! mode runs the original alone. Self-test: a copy whose calls go to stand-ins, against the
//! Rust version, the session on or off, the lock failing or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::connecting::status;
use crate::net::lobby::{Part, SESSION, STOP, g, lobby_entry};

const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background

/// The Rust version: the registers it leaves (the entry's).
pub fn stopping(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(SESSION) == 1 {
        part(SHOW_BACKGROUND, &mut r)?;
        status(&mut r, 0x157, part)?;
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
    lobby_entry(r, stopping)
}
