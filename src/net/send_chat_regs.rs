//! `net::send_chat_regs` (0x41819B): `net::send_chat` inside `pusha` .. `popa`.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept; the flags `net::send_chat` leaves (no caller reads them)
//! Caller: `net::chat_key`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, lobby_entry};

const SEND_CHAT: u32 = 0x4193C8; // net::send_chat

/// The Rust version: the registers it leaves (the entry's).
pub fn send_chat_regs(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(SEND_CHAT, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, send_chat_regs)
}
