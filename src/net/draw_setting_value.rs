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
use crate::net::lobby::{Cursor, EDITED, FONT, TEXT, a, g};
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;
use rc2k_core::net::screens::{self, Value};

/// The Rust version: the text at `text` for item `item` at `y` (x 0x190), edited (with the
/// caret), nothing (the "nothing" string) or as it is; returns the next y (y + 0x1E, the
/// original's ebp). The registers it leaves are read by no caller (`settings_frame`,
/// `draw_number_value`).
pub fn draw_setting_value(text: u32, item: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    let shown =
        match screens::text_value(item == g(EDITED), || unsafe { (text as *const u8).read() }) {
            Value::Edited => {
                let mut cursor = Cursor::text();
                cursor.string(text);
                cursor.string_id(screens::CARET);
                cursor.nul();
                a(TEXT)
            }
            Value::Nothing => {
                let mut cursor = Cursor::text();
                cursor.string_id(screens::NOTHING);
                cursor.nul();
                a(TEXT)
            }
            Value::AsIs => text,
        };
    unsafe {
        crate::text::draw_text::draw_text_at(
            a(FONT) as *mut Font,
            shown as *const u8,
            0x190,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(y.wrapping_add(0x1E))
}
