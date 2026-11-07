//! `text::measure_text` (0x40230B): runs `draw_text` with the font's two result fields zeroed
//! first, and returns them.
//!
//! In:  eax = font, ebx = text, ecx = x
//! Out: carry and eax = 0 as `draw_text` leaves them; ebx = the font's dword at +0x14,
//!      ecx = the one at +0x1C, edx = `draw_text`'s y; esi, edi, ebp preserved
//! With a font in measure mode nothing is drawn, +0x14 counts the lines and +0x1C is the
//! width of the widest one, which is what its twelve callers want. The y passed on is the
//! font's dword at +8. With any other font it would draw, with a null palette: reproduced as
//! it is.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::gfx::glyph_store;
use crate::pointers::GamePtr;
use crate::text::draw_text::{Adapt, Canvas, Outcome, refusal};
use crate::text::font::Font;
use rc2k_core::strings::bytes::Bytes;

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measured {
    pub outcome: Outcome,
    /// The font's dword at +0x14 afterwards: the number of lines, in measure mode.
    pub lines: u32,
    /// The font's dword at +0x1C afterwards: the widest line, in measure mode.
    pub width: u32,
}

/// `font` is changed as the original changes the font object. Converted:
/// `rc2k_core::text::measure`.
pub fn measure_text(
    font: &mut Font,
    text: &(impl Bytes + ?Sized),
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let glyphs = glyph_store::bytes(font.glyph_table, "a font's glyph table")?;
    let m = refusal(rc2k_core::text::measure(
        font,
        text,
        &glyphs,
        x,
        &mut Adapt::new(canvas),
    ))?;
    Ok(Measured {
        outcome: Outcome {
            too_wide: m.outcome.too_wide,
            x: m.outcome.x,
            y: m.outcome.y,
        },
        lines: m.lines,
        width: m.width,
    })
}

/// The font `font`, refused unless it only measures (`MODE_MEASURE`): then a measured width
/// does not depend on the x the measuring starts from, so callers whose original hands
/// whatever x its caller left measure from 0. A font in another mode would draw the text.
pub fn only_measures(
    font: crate::gfx::typefaces::FontId,
) -> Result<GamePtr, crate::errors::BadInput> {
    if font.object().mode.get() != rc2k_core::text::MODE_MEASURE {
        return Err(crate::errors::BadInput("a measuring font that draws"));
    }
    Ok(font.game_ptr())
}

/// For Rust callers, with a font object in game memory: the object is only written if the
/// canvas keeps font changes.
pub fn measure_text_at(
    font: GamePtr,
    text: &(impl Bytes + ?Sized),
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let mut copy = font.read::<Font>("a font")?;
    let measured = measure_text(&mut copy, text, x, canvas)?;
    if canvas.keeps_font_changes() {
        font.write(copy, "a font")?;
    }
    Ok(measured)
}
