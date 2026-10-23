//! `net::chat_input_line` (0x418083): the lobby's chat input drawn (`text::draw_text` with
//! `stru_51E584` at x 0xA, y ebp; ebp 0x1E lower), with string 4 (the cursor) after it while
//! it is being edited (`dword_52A4B8` the item), built in the game's text buffer.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them; ebp + 0x1E; no `pusha`
//! Caller: `net::lobby_frame`. Its calls through `part` (`strings::append_nul` inline).
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version, edited or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    APPEND_STRING, DRAW_TEXT, EDITED, Part, STR_APPEND, TEXT, a, g, lobby_entry,
};

pub const CHAT_FONT: usize = 0x51E584; // stru_51E584

/// The Rust version: the registers it leaves.
pub fn chat_input_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.esi == g(EDITED) {
        r.ebx = a(TEXT);
        part(STR_APPEND, &mut r)?;
        r.eax = 4;
        part(APPEND_STRING, &mut r)?;
        unsafe { (r.ebx as *mut u8).write(0) };
        r.ebx = r.ebx.wrapping_add(1);
        r.eax = a(TEXT);
    }
    r.ebx = r.eax;
    r.eax = a(CHAT_FONT);
    r.ecx = 0xA;
    r.edx = r.ebp;
    r.ebp = r.ebp.wrapping_add(0x1E);
    part(DRAW_TEXT, &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, chat_input_line)
}
