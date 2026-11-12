//! `net::draw_setting_value` (0x416BC4): a setting's value drawn (`text::draw_text` with
//! `font` at 0x190, at y; y 0x1E lower): for the item being edited (`edited`) the
//! text with string 4 (the cursor) after it, built in the game's text buffer; an empty text
//! as string 0x16E; any other as it is.
//!
//! Takes the text, the item and y.
//! Callers: `net::settings_frame`, `net::draw_number_value` (which goes on into it).

use crate::errors::Refused;
use crate::net::lobby::{FONT, push_id, store_text};
use crate::text::draw_text::GameCanvas;
use rc2k_core::net::screens::{self, Value};

/// The text `text` (up to its NUL) for item `item` at `y` (x 0x190),
/// edited (with the caret), nothing (the "nothing" string) or as it is; the first two are
/// built in the game's text buffer. Returns the next y (y + 0x1E).
pub fn draw_setting_value(
    o: crate::owners::Owners<'static>,
    text: &[u8],
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    let net = o.net;
    // Converted: `rc2k_core::net::screens` (`text_value`, `Value`, `CARET`, `NOTHING`).
    let value = screens::text_value(item == net.edit().edited.get(), || {
        text.first().copied().unwrap_or(0)
    });
    let shown = match value {
        Value::Edited => {
            let mut built = text.to_vec();
            push_id(o, &mut built, screens::CARET)?;
            store_text(o.strings, built)?
        }
        Value::Nothing => {
            let mut built = Vec::new();
            push_id(o, &mut built, screens::NOTHING)?;
            store_text(o.strings, built)?
        }
        Value::AsIs => {
            let mut plain = text.to_vec();
            plain.push(0);
            plain
        }
    };
    crate::text::draw_text::draw_text_at(
        FONT,
        &shown,
        0x190,
        y as i32,
        &mut GameCanvas::new(o.gfx),
    )?;
    Ok(y.wrapping_add(0x1E))
}
