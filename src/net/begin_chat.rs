//! `net::begin_chat` (0x4180C5): Enter on the lobby's chat item starts a message: the input
//! (`byte_52A4C8`) emptied and edited (`net::begin_edit`, the item `g_listChoice`, width
//! 0x30C, measured with `dword_51E424`).
//!
//! In:  nothing
//! Out: eax = the item, ebx = the input, ecx = 0x30C, edx = `dword_51E424`'s address; no
//!      `pusha`
//! Caller: `net::lobby_loop`. Its call through `part`. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{CHAT_INPUT, CHOICE, Part, a, g, lobby_entry, pb};

const BEGIN_EDIT: u32 = 0x416E1A; // net::begin_edit
const CHAT_FONT: usize = 0x51E424; // dword_51E424

/// The Rust version: the registers it leaves.
pub fn begin_chat(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(CHOICE);
    r.ebx = a(CHAT_INPUT);
    pb(CHAT_INPUT, 0);
    r.ecx = 0x30C;
    r.edx = a(CHAT_FONT);
    part(BEGIN_EDIT, &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, begin_chat)
}
