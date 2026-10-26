//! `net::draw_setting_value` (0x416BC4): a setting's value drawn (`text::draw_text` with
//! `font` at 0x190, y ebp; ebp 0x1E lower): for the item being edited (`dword_52A4B8`) the
//! text with string 4 (the cursor) after it, built in the game's text buffer; an empty text
//! as string 0x16E; any other as it is.
//!
//! In:  eax = the text, esi = the item, ebp = y
//! Out: as `text::draw_text` leaves them (eax 0, ebx past the text, ecx, edx); ebp + 0x1E;
//!      no `pusha`
//! Callers: `net::settings_frame`, `net::draw_number_value` (which goes on into it).

use crate::errors::Refused;
use crate::net::lobby::{Cursor, FONT};
use crate::net::store::NET;
use crate::pointers::GamePtr;
use crate::strings::text_buffer::TEXT_BUFFER;
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::screens::{self, Value};

/// The Rust version: the text at `text` for item `item` at `y` (x 0x190), edited (with the
/// caret), nothing (the "nothing" string) or as it is; returns the next y (y + 0x1E, the
/// original's ebp). The registers it leaves are read by no caller (`settings_frame`,
/// `draw_number_value`).
pub fn draw_setting_value(text: u32, item: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    // The core asks for the first byte only when it needs it: a refusal is kept and raised.
    let refused = std::cell::Cell::new(None);
    let value = screens::text_value(item == NET.edited.get(), || {
        GamePtr::new(text)
            .read::<u8>("a setting's text")
            .unwrap_or_else(|e| {
                refused.set(Some(e));
                0
            })
    });
    if let Some(e) = refused.take() {
        return Err(e.into());
    }
    let shown = match value {
        Value::Edited => {
            let mut cursor = Cursor::text();
            cursor.string(text);
            cursor.string_id(screens::CARET);
            cursor.nul()?;
            TEXT_BUFFER.text.addr() as u32
        }
        Value::Nothing => {
            let mut cursor = Cursor::text();
            cursor.string_id(screens::NOTHING);
            cursor.nul()?;
            TEXT_BUFFER.text.addr() as u32
        }
        Value::AsIs => text,
    };
    unsafe {
        crate::text::draw_text::draw_text_at(
            FONT.as_ptr(),
            shown as *const u8,
            0x190,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(y.wrapping_add(0x1E))
}
