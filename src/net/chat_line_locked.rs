//! `net::chat_line_locked` (0x4193FF): `net::add_chat_line` with the received messages' lock
//! held (`net::lock_received` .. `net::unlock_received`): a chat message another player sent.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: the receiving thread's message handling (0x41959x, not ported). Its calls through
//! `part`. Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{LOCK_RECEIVED, Part, UNLOCK_RECEIVED, lobby_entry};

const ADD_CHAT_LINE: u32 = 0x419428; // net::add_chat_line

/// The Rust version: the registers it leaves (the entry's).
pub fn chat_line_locked(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(LOCK_RECEIVED, &mut r)?;
    part(ADD_CHAT_LINE, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, chat_line_locked)
}
