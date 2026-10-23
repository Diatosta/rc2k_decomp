//! `net::setting_key` (0x416C76): a key for the setting being edited (`net::edit_key`); when
//! that ends the editing: the player's name (item 1) left empty gets its old text back
//! (`byte_95EAF8`, 64 bytes); the most players (item 2) read from its text
//! (`settings::parse_number`), held to 2 .. 8 (signed), and to 4 with `g_noticeKind` 2, into
//! `dword_52ABD0`.
//!
//! In:  al = the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it, eax = the item (or
//!      the number), the copy's esi, edi, ecx; no `pusha`; the flags of the last test (no
//!      caller reads them)
//! Caller: `net::settings_loop`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins (the editing ended or not, the
//! number given), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::lobby::{
    CHOICE, EDIT_SAVED, MACHINE_NAME, MOST_PLAYERS, NOTICE_KIND, NUMBER_TEXT, PARSE_NUMBER, Part,
    a, g, gb, lobby_entry, movsb, p,
};
use rc2k_core::net::chat;

const EDIT_KEY: u32 = 0x416E43; // net::edit_key

/// The Rust version: the registers it leaves.
pub fn setting_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(EDIT_KEY, &mut r)?;
    if r.eflags & ZERO_FLAG == 0 {
        return Ok(r);
    }
    r.eax = g(CHOICE);
    // Converted: `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match r.eax {
        chat::NAME => {
            r.esi = a(EDIT_SAVED);
            r.edi = a(MACHINE_NAME);
            if gb(MACHINE_NAME) == 0 {
                movsb(r.edi, r.esi, 0x40);
                r.esi += 0x40;
                r.edi += 0x40;
                r.ecx = 0;
            }
        }
        chat::MOST_PLAYERS => {
            r.eax = a(NUMBER_TEXT);
            part(PARSE_NUMBER, &mut r)?;
            r.eax = chat::most_players(r.eax, gb(NOTICE_KIND));
            p(MOST_PLAYERS, r.eax);
        }
        _ => {}
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, setting_key)
}
