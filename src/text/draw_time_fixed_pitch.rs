//! Draws the first eight bytes of a text one at a time in fixed columns, so that the digits of a
//! time like "01:23.45" do not move.
//! Each byte is copied to text + 0x40 with a NUL after it and drawn from there, so the text needs
//! 0x42 bytes of room. All eight bytes are read even if a NUL comes earlier. The columns are at x -
//! 100 plus 0, 13, 23, 33, 46, 56, 66 and 79.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::Font;
use rc2k_core::strings::text::{TIME_STEPS, columns};

/// Where the one-character text is built, relative to the text.
pub const CELL_OFFSET: usize = 0x40;

/// Returns whether the last character was too wide.
pub fn draw_time_fixed_pitch(
    font: &mut Font,
    text: crate::strings::text_buffer::TextBuf,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let mut too_wide = false;
    for (i, x) in columns(x, &TIME_STEPS).into_iter().enumerate() {
        let cell = put_cell(text, i)?;
        too_wide = draw_text_keep_regs(font, &cell, x, y, canvas)?;
    }
    Ok(too_wide)
}

/// Copies the text's byte `i` to its cell with a NUL after it and returns the cell, which is what
/// is drawn.
pub fn put_cell(text: crate::strings::text_buffer::TextBuf, i: usize) -> Result<[u8; 2], Refused> {
    let b = text.byte(i, "a fixed-pitch text")?;
    let cell = [b, 0];
    text.write_at(CELL_OFFSET, &cell, "a fixed-pitch text's cell")?;
    Ok(cell)
}

/// The same with a font object in game memory, for Rust callers.
pub fn draw_time_fixed_pitch_at(
    font: impl Into<crate::gfx::typefaces::FontRef>,
    text: crate::strings::text_buffer::TextBuf,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let font = font.into().object(canvas.typefaces());
    let before = font.get();
    let mut copy = before;
    let too_wide = draw_time_fixed_pitch(&mut copy, text, x, y, canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(too_wide)
}
