//! `text::draw_string_by_id` (0x40234A): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and draws it with `draw_text`.
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y
//! Out: as `draw_text` leaves it: carry, eax = 0, ebx = position in the text buffer, ecx = x,
//!      edx = y; esi, edi and ebp preserved
//! A text longer than the buffer is refused: the original writes past it.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::text_from_ids::{TEXT_LEN, text_at};
use crate::text::draw_text::{Canvas, Outcome, draw_text};
use crate::text::font::Font;

// ---- Rust version ----

/// The text of string `id` with its NUL, as the original leaves it in the text buffer.
pub fn string_text(id: u32) -> Result<Vec<u8>, BadInput> {
    let mut out = Vec::new();
    append_string_by_id(id, &mut |bytes| out.extend_from_slice(bytes))?;
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
    id: u32,
    font: &mut Font,
    x: i32,
    y: i32,
    buffer: GamePtr,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let bytes = string_text(id)?;
    buffer.write_bytes(&bytes, "the text buffer")?;
    draw_text(font, &bytes, x, y, canvas)
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
pub fn draw_string_by_id_at(
    id: u32,
    font: GamePtr,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let before = font.read::<Font>("a font")?;
    let mut copy = before;
    let out = draw_string_by_id(id, &mut copy, x, y, text_at(), canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.write(copy, "a font")?;
    }
    Ok(out)
}
