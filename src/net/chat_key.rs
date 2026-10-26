//! `net::chat_key` (0x4180E4): a key for the chat message being typed (`net::edit_key`); when
//! that ends the editing and the message is not empty, it is sent (`net::send_chat_regs`,
//! this machine's player) and the input emptied.
//!
//! In:  al = the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it eax = the player,
//!      ebx = the input; no `pusha`
//! Caller: `net::lobby_loop`. Its calls through `part`. Shadow mode runs the original alone
//! (it sends). Self-test: a copy whose calls go to stand-ins, against the Rust version:
//! editing on, ended with a message, ended empty.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::lobby::{CHAT_INPUT, PLAYER, Part, a, g, gb, lobby_entry, pb};

const EDIT_KEY: u32 = 0x416E43; // net::edit_key
const SEND_CHAT: u32 = 0x41819B; // net::send_chat_regs

/// The Rust version: the registers it leaves.
pub fn chat_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(EDIT_KEY, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(r);
    }
    r.eax = g(PLAYER);
    r.ebx = a(CHAT_INPUT);
    if gb(CHAT_INPUT) != 0 {
        part(SEND_CHAT, &mut r)?;
        pb(CHAT_INPUT, 0);
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, chat_key)
}
