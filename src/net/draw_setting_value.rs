//! `net::draw_setting_value` (0x416BC4): a setting's value drawn (`text::draw_text` with
//! `font` at 0x190, y ebp; ebp 0x1E lower): for the item being edited (`dword_52A4B8`) the
//! text with string 4 (the cursor) after it, built in the game's text buffer; an empty text
//! as string 0x16E; any other as it is.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them (eax 0, ebx past the text, ecx, edx); ebp + 0x1E;
//!      no `pusha`
//! Callers: `net::settings_frame`, `net::draw_number_value` (which goes on into it). Its
//! calls through `part` (`strings::append_nul` inline). Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose calls go to stand-ins (the strings appended), against
//! the Rust version: edited, empty, a text.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    APPEND_STRING, DRAW_TEXT, EDITED, FONT, Part, STR_APPEND, TEXT, a, g, lobby_entry,
};
use rc2k_core::net::screens::{self, Value};

fn nul(r: &mut Regs) {
    unsafe { (r.ebx as *mut u8).write(0) };
    r.ebx = r.ebx.wrapping_add(1);
}

/// The Rust version: the registers it leaves.
pub fn draw_setting_value(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    let text = r.eax;
    match screens::text_value(r.esi == g(EDITED), || unsafe { (text as *const u8).read() }) {
        Value::Edited => {
            r.ebx = a(TEXT);
            part(STR_APPEND, &mut r)?;
            r.eax = screens::CARET;
            part(APPEND_STRING, &mut r)?;
            nul(&mut r);
            r.eax = a(TEXT);
        }
        Value::Nothing => {
            r.eax = screens::NOTHING;
            r.ebx = a(TEXT);
            part(APPEND_STRING, &mut r)?;
            nul(&mut r);
            r.eax = a(TEXT);
        }
        Value::AsIs => {}
    }
    r.ebx = r.eax;
    r.eax = a(FONT);
    r.ecx = 0x190;
    r.edx = r.ebp;
    r.ebp = r.ebp.wrapping_add(0x1E);
    part(DRAW_TEXT, &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, draw_setting_value)
}
