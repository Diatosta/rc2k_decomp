//! `net::send_chat` (0x4193C8): this player's chat message added to the chat
//! (`net::add_chat_line`, under the received messages' lock) and sent to everyone
//! (`g_netSendTo`, DirectPlay's `Send` through `net::dplay_send`) as '"' and the text, no NUL.
//!
//! In:  eax = the player, ebx = the message
//! Out: every register kept (`pusha` .. `popa`); the flags of `test eax, eax` on the send's
//!      eax (no caller reads them)
//! Caller: `net::send_chat_regs`. Its calls through `part` (`net::lobby_calls`: the Rust
//! versions, the send through the game). Shadow mode runs the original alone (it sends).
//! Self-test: a copy whose calls go to stand-ins (`str_append` done), against the Rust
//! version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    LOCK_RECEIVED, Part, SEND, SEND_BUFFER, SEND_TO, STR_APPEND, UNLOCK_RECEIVED, a, g,
    lobby_entry, test_eax,
};

const ADD_CHAT_LINE: u32 = 0x419428; // net::add_chat_line

/// The Rust version: the registers it leaves (the entry's).
pub fn send_chat(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(LOCK_RECEIVED, &mut r)?;
    part(ADD_CHAT_LINE, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    r.eax = r.ebx;
    let buffer = a(SEND_BUFFER);
    unsafe { (buffer as *mut u8).write(b'"') };
    r.ebx = buffer + 1;
    part(STR_APPEND, &mut r)?;
    r.ecx = r.ebx.wrapping_sub(buffer);
    r.ebx = buffer;
    r.eax = g(SEND_TO);
    r.edx = 0;
    part(SEND, &mut r)?;
    test_eax(&mut r);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, send_chat)
}
