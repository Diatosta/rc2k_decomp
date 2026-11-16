//! Like `draw_time_fixed_pitch`, for a time of day: the first five bytes of a text ("09:05") one at
//! a time in fixed columns, at x - 100 plus 0, 13, 23, 33 and 46.
//! Each byte is copied to text + 0x40 with a NUL after it and drawn from there.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::Font;
use rc2k_core::strings::text::{CLOCK_STEPS, columns};

/// Returns (carry, zero) of the last addition, which callers read.
pub fn draw_clock_fixed_pitch(
    font: &mut Font,
    text: crate::strings::text_buffer::TextBuf,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), Refused> {
    let at = columns(x, &CLOCK_STEPS);
    for (i, &x) in at.iter().enumerate() {
        let cell = crate::text::draw_time_fixed_pitch::put_cell(text, i)?;
        draw_text_keep_regs(font, &cell, x, y, canvas)?;
    }
    // The flags of the last addition, after the fifth character.
    let (sum, carry) = (at[4] as u32).overflowing_add(CLOCK_STEPS[4] as u32);
    Ok((carry, sum == 0))
}

/// The same with a font object in game memory, for Rust callers.
pub fn draw_clock_fixed_pitch_at(
    font: impl Into<crate::gfx::typefaces::FontRef>,
    text: crate::strings::text_buffer::TextBuf,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), Refused> {
    let font = font.into().object(canvas.typefaces());
    let before = font.get();
    let mut copy = before;
    let too_wide = draw_clock_fixed_pitch(&mut copy, text, x, y, canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(too_wide)
}
