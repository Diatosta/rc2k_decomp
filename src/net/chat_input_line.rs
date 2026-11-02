//! `net::chat_input_line` (0x418083): the lobby's chat input drawn (`text::draw_text` with
//! `stru_51E584` at x 0xA, y ebp; ebp 0x1E lower), with string 4 (the cursor) after it while
//! it is being edited (`dword_52A4B8` the item), built in the game's text buffer.
//!
//! In:  the text, the item, y
//! Out: as `text::draw_text` leaves them; ebp + 0x1E; no `pusha`
//! Caller: `net::lobby_frame`.

use crate::errors::Refused;
use crate::net::lobby::Cursor;
use crate::net::store::NET;
use crate::strings::text_buffer::TEXT_BUFFER;
use crate::text::draw_text::GameCanvas;

pub const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NAME;

/// The Rust version: the chat's input `text` for item `item` at `y` (x 0xA), with the caret
/// while it is the item being edited; returns the next y (y + 0x1E, the original's ebp).
pub fn chat_input_line(
    text: impl Into<crate::place::Place>,
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    let text = text.into();
    chat_input_line_at(text, item, y)
}

fn chat_input_line_at(text: crate::place::Place, item: u32, y: u32) -> Result<u32, Refused> {
    let shown = if item == NET.edited.get() {
        let mut cursor = Cursor::text();
        cursor.string(text);
        cursor.string_id(4);
        cursor.nul()?;
        crate::place::Place::from(&TEXT_BUFFER.text)
    } else {
        text
    };
    {
        crate::text::draw_text::draw_text_at(
            CHAT_FONT.game_ptr(),
            shown.game_ptr(),
            0xA,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(y.wrapping_add(0x1E))
}
