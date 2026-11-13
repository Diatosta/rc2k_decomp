//! Builds the text of a string id in the shared text buffer, NUL-terminates it and measures it with
//! `measure_text` (the font is one in measure mode). A text longer than the buffer is refused.

use crate::errors::Refused;
use crate::gfx::typefaces::FontRef;
use crate::strings::append_string_by_id::StringOwners;
use crate::strings::text_buffer::TextBuf;
use crate::strings::text_from_ids::text_at;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_text::{Measured, measure_text};

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes. `font` is updated as `measure_text` updates it.
pub fn measure_string_by_id(
    so: StringOwners<'_>,
    id: u32,
    font: &mut Font,
    x: i32,
    buffer: TextBuf,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let bytes = string_text(so, id)?;
    buffer.write_at(0, &bytes, "the text buffer")?;
    measure_text(font, &bytes, x, canvas)
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers. The
/// font object is only written if the canvas keeps font changes.
pub fn measure_string_by_id_at(
    so: StringOwners<'static>,
    id: u32,
    font: impl Into<FontRef>,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let font = font.into().object(canvas.typefaces());
    let mut copy = font.get();
    let measured = measure_string_by_id(so, id, &mut copy, x, text_at(so.strings), canvas)?;
    if canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(measured)
}

pub const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;
