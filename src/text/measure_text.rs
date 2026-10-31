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
use crate::pointers::GamePtr;
use crate::text::draw_text::{Canvas, Outcome, outcome, run_core};
use crate::text::font::Font;

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
    text: GamePtr,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    let copy = *font;
    let m = {
        run_core(&copy, text, canvas, |words, glyphs, c| {
            rc2k_core::text::measure(font, words, glyphs, x, c)
        })
    }?;
    Ok(Measured {
        outcome: outcome(text, m.outcome),
        lines: m.lines,
        width: m.width,
    })
}

/// `measure_text_at` on a text of Rust's own (an owned block's): its widest line's width
/// and its line count.
pub fn measure_bytes_at(
    font: GamePtr,
    text: &[u8],
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::{Adapt, AdaptError};
    let at = font;
    let mut copy = at.read::<Font>("a font")?;
    let glyphs = crate::gfx::glyph_store::bytes(copy.glyph_table, "a font's glyph table")?;
    let m = match rc2k_core::text::measure(&mut copy, text, &glyphs, x, &mut Adapt::new(canvas)) {
        Ok(m) => m,
        Err(AdaptError::Refused(e)) => return Err(e),
        Err(AdaptError::Invalid) => {
            return Err(
                crate::errors::BadInput("a read past a text or a font's glyph table").into(),
            );
        }
    };
    if canvas.keeps_font_changes() {
        at.write(copy, "a font")?;
    }
    Ok((m.width, m.lines))
}

/// For Rust callers, with a font object in game memory: the object is only written if the
/// canvas keeps font changes.
pub fn measure_text_at(
    font: GamePtr,
    text: GamePtr,
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
