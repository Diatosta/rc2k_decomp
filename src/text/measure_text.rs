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

use crate::errors::FatalError;
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
pub unsafe fn measure_text(
    font: &mut Font,
    text: *const u8,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, FatalError> {
    let copy = *font;
    let m = unsafe {
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

/// For Rust callers, with a font object in game memory: the object is only written if the
/// canvas keeps font changes.
pub unsafe fn measure_text_at(
    font: *mut Font,
    text: *const u8,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, FatalError> {
    unsafe {
        let mut copy = font.read_unaligned();
        let measured = measure_text(&mut copy, text, x, canvas)?;
        if canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(measured)
    }
}
