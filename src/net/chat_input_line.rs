//! `net::chat_input_line` (0x418083): the lobby's chat input drawn (`text::draw_text` with
//! `typefaces::NAME` at x 0xA, one line lower each time), with string 4 (the cursor) after it while
//! it is being edited (`edited` the item), built in the game's text buffer.
//!
//! Takes the text, the item and y.
//! Caller: `net::lobby_frame`.

use crate::errors::Refused;
use crate::net::lobby::{push_id, store_text};
use crate::net::owner::net;
use crate::text::draw_text::GameCanvas;

pub const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NAME;

/// The chat's input `text` (up to its NUL) for item `item` at `y` (x 0xA),
/// with the caret while it is the item being edited (built in the game's text buffer);
/// returns the next y (y + 0x1E).
pub fn chat_input_line(text: &[u8], item: u32, y: u32) -> Result<u32, Refused> {
    let shown = if item == net().edit().edited.get() {
        let mut built = text.to_vec();
        push_id(&mut built, 4)?;
        store_text(built)?
    } else {
        let mut plain = text.to_vec();
        plain.push(0);
        plain
    };
    crate::text::draw_text::draw_text_at(CHAT_FONT, &shown, 0xA, y as i32, &mut GameCanvas)?;
    Ok(y.wrapping_add(0x1E))
}
