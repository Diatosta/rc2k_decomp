//! `text::draw_string_by_id` (0x40234A): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and draws it with `draw_text`.
//!
//! Takes: string id, font, x, y
//! A text longer than the buffer is refused: the original writes past it.

use crate::errors::{BadInput, Refused};
use crate::gfx::typefaces::FontRef;
use crate::strings::append_string_by_id::{StringOwners, append_string_by_id};
use crate::strings::text_buffer::TextBuf;
use crate::strings::text_from_ids::{TEXT_LEN, text_at};
use crate::text::draw_text::{Canvas, Outcome, draw_text};
use crate::text::font::Font;

/// The text of string `id` with its NUL, as the original leaves it in the text buffer.
pub fn string_text(o: StringOwners<'_>, id: u32) -> Result<Vec<u8>, BadInput> {
    let mut out = Vec::new();
    append_string_by_id(o, id, &mut |bytes| out.extend_from_slice(bytes))?;
    out.push(0);
    if out.len() > TEXT_LEN {
        return Err(BadInput(
            "text longer than the text buffer: the original writes past it",
        ));
    }
    Ok(out)
}

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes when the game's must not be touched.
pub fn draw_string_by_id(
    o: StringOwners<'_>,
    id: u32,
    font: &mut Font,
    x: i32,
    y: i32,
    buffer: TextBuf,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let bytes = string_text(o, id)?;
    buffer.write_at(0, &bytes, "the text buffer")?;
    draw_text(font, &bytes, x, y, canvas)
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
pub fn draw_string_by_id_at(
    o: StringOwners<'static>,
    id: u32,
    font: impl Into<FontRef>,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let font = font.into().object(canvas.typefaces());
    let before = font.get();
    let mut copy = before;
    let out = draw_string_by_id(o, id, &mut copy, x, y, text_at(o.strings), canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(out)
}
