//! Draws the lobby's chat input line, followed by the cursor string while it is being edited.

use crate::errors::Refused;
use crate::net::lobby::{push_id, store_text};
use crate::text::draw_text::GameCanvas;

pub const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::NAME;

/// The chat's input `text` (up to its NUL) for item `item` at `y` (x 0xA),
/// with the caret while it is the item being edited (built in the game's text buffer);
/// returns the next y (y + 0x1E).
pub fn chat_input_line(
    o: crate::owners::Owners<'static>,
    text: &[u8],
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    let net = o.net;
    let shown = if item == net.lobby().edit.edited.get() {
        let mut built = text.to_vec();
        push_id(o, &mut built, 4)?;
        store_text(o.strings, built)?
    } else {
        let mut plain = text.to_vec();
        plain.push(0);
        plain
    };
    crate::text::draw_text::draw_text_at(
        CHAT_FONT,
        &shown,
        0xA,
        y as i32,
        &mut GameCanvas::new(o.gfx),
    )?;
    Ok(y.wrapping_add(0x1E))
}
