//! `net::clear_chat` (0x419411): the lobby's chat (`byte_52A508`) made eight empty lines:
//! seven 0x0A and a NUL.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `net::start_link` (with the received messages' lock held). Shadow mode: both run,
//! the eight bytes compared. Self-test: a copy against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{CHAT, Part, a};

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_chat(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::chat::EMPTY`.
    unsafe { (a(CHAT) as *mut [u8; 8]).write_unaligned(rc2k_core::net::chat::EMPTY) };
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| clear_chat(r, &mut crate::net::lobby::game_part))
}
