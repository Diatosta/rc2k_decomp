//! `text::draw_time_text` (0x402395): draws a time's text one character at a time, each
//! with the font's fixed width (+0x1C) set to 6 for ':' and '.', 0x10 for the others, so
//! that the digits keep their columns; at most 11 characters, up to the NUL.
//!
//! Takes: font, text, x, y
//!
//! Each character is drawn as a text of its own (the original swapped the dword after it
//! for 0 around `text::draw_text_keep_regs` and put it back: here the character and a NUL
//! are drawn as a value and the buffer is not touched); x moves on by the width. The font's +0x1C is 0 at the end. Callers: `net::lobby_frame` (twice).

use crate::errors::Refused;

/// The time text at `text` in `font` at (`x`, `y`).
/// The flags the original leaves are read by neither caller.
pub fn draw_time_text(
    font: impl Into<crate::gfx::typefaces::FontRef>,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    let font = font.into().object(&gfx.fonts.typefaces);
    let mut x = x;
    // Converted: `rc2k_core::text::time_text_widths`.
    let chars = text;
    let widths = rc2k_core::text::time_text_widths(chars)
        .map_err(|_| crate::errors::BadInput("a time text past its block"))?;
    for (i, width) in widths.into_iter().enumerate() {
        font.box_width.set(width);
        // `text::draw_text_keep_regs` on a copy of the font, written back if it changed.
        let mut copy = font.get();
        crate::text::draw_text_keep_regs::draw_text_keep_regs(
            &mut copy,
            &[chars.byte(i).unwrap_or(0), 0],
            x as i32,
            y as i32,
            &mut GameCanvas::new(gfx),
        )?;
        if copy != font.get() {
            font.set(copy);
        }
        x = x.wrapping_add(width);
    }
    font.box_width.set(0);
    Ok(())
}
