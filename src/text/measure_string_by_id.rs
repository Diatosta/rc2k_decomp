//! `text::measure_string_by_id` (0x40232C): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and measures it with `measure_text`.
//!
//! Takes: string id, font (one in measure mode), x
//! Out: as `measure_text`: ebx = lines, ecx = the widest line, edx = y, eax = 0, carry;
//!      esi, edi, ebp preserved
//! 13 callers. A text longer than the buffer is refused: the original writes past it.

use crate::errors::Refused;
use crate::gfx::typefaces::FontRef;
use crate::pointers::GamePtr;
use crate::strings::text_from_ids::text_at;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_text::{Measured, measure_text};

// ---- Rust version ----

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes. `font` is changed as the original changes the font object.
pub fn measure_string_by_id(
    id: u32,
    font: &mut Font,
    x: i32,
    buffer: GamePtr,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let bytes = string_text(id)?;
    buffer.write_bytes(&bytes, "the text buffer")?;
    measure_text(font, &bytes, x, canvas)
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
/// The font object is only written if the canvas keeps font changes.
pub fn measure_string_by_id_at(
    id: u32,
    font: impl Into<FontRef>,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let font = font.into().object();
    let mut copy = font.get();
    let measured = measure_string_by_id(id, &mut copy, x, text_at(), canvas)?;
    if canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(measured)
}

pub const MEASURING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE;
