//! `net::chat_input_line` (0x418083): the lobby's chat input drawn (`text::draw_text` with
//! `stru_51E584` at x 0xA, y ebp; ebp 0x1E lower), with string 4 (the cursor) after it while
//! it is being edited (`dword_52A4B8` the item), built in the game's text buffer.
//!
//! In:  the text, the item, y
//! Out: as `text::draw_text` leaves them; ebp + 0x1E; no `pusha`
//! Caller: `net::lobby_frame`.

use crate::errors::Refused;
use crate::net::lobby::{Cursor, EDITED, TEXT, a, g};
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;

pub const CHAT_FONT: usize = 0x51E584; // stru_51E584

/// The Rust version: the chat's input `text` for item `item` at `y` (x 0xA), with the caret
/// while it is the item being edited; returns the next y (y + 0x1E, the original's ebp).
pub fn chat_input_line(text: u32, item: u32, y: u32) -> Result<u32, Refused> {
    let shown = if item == g(EDITED) {
        let mut cursor = Cursor::text();
        cursor.string(text);
        cursor.string_id(4);
        cursor.nul();
        a(TEXT)
    } else {
        text
    };
    unsafe {
        crate::text::draw_text::draw_text_at(
            a(CHAT_FONT) as *mut Font,
            shown as *const u8,
            0xA,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(y.wrapping_add(0x1E))
}
