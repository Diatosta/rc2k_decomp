//! `text::draw_text` (0x4023D5): draws NUL-terminated text with a bitmap font, line by line.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: carry set if a line was wider than the box (`FLAG_FAIL_IF_WIDER`), else clear;
//!      eax = 0, ebx = text after the NUL (the start of the failed line on carry),
//!      ecx = x as given, edx = y of the last line; esi, edi and ebp preserved
//!
//! Bytes from `first_char` to `last_char` are glyphs, bytes above are ignored, bytes below are
//! control codes:
//!   0x00        end of text
//!   0x0A        new line: back to x, down by the line height
//!   0x0E n      x += n (signed byte)
//!   0x0F n      y += n (signed byte); ignored while measuring
//!   0x10.. n    extended glyph (((code - 0x10) << 8 | n) >> 1) + number of normal glyphs
//!   other       x -= code
//! Except in `MODE_AT_X` each line is measured before it is drawn. In `MODE_MEASURE` nothing
//! is drawn and the font object is updated instead.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::gfx::blit_glyph::{BlitCall, blit_glyph_on_surface};
use crate::gfx::glyph_store::{self, Glyphs};
use crate::pointers::{GameBytes, GamePtr};
use crate::text::font::Font;
use crate::text::glyph_kerning::{State, state_ptr};

// ---- Rust version ----

/// What drawing text needs from outside: the kerning row table and the blitter.
pub trait Canvas {
    /// False for a canvas that only predicts: font objects in game memory are then left alone.
    fn keeps_font_changes(&self) -> bool;
    /// The kerning row table the canvas keeps.
    fn kerning(&mut self) -> &mut State;
    fn blit_glyph(&mut self, call: BlitCall) -> Result<(), Refused>;
}

/// The game's own kerning table and surface.
pub struct GameCanvas;

impl Canvas for GameCanvas {
    fn keeps_font_changes(&self) -> bool {
        true
    }
    fn kerning(&mut self) -> &mut State {
        unsafe { &mut *state_ptr() }
    }
    fn blit_glyph(&mut self, c: BlitCall) -> Result<(), Refused> {
        blit_glyph_on_surface(c.glyph, c.table, c.x, c.y, c.palette).map(|_| ())
    }
}

/// How `draw_text` ended. The original also returns eax = 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Outcome {
    /// A line was wider than the box and the font has `FLAG_FAIL_IF_WIDER`: the carry flag.
    pub too_wide: bool,
    /// After the NUL, or the start of the line that was too wide: ebx.
    pub text: *const u8,
    /// The x that was passed in: ecx.
    pub x: i32,
    /// The y of the last line, font offset included: edx.
    pub y: i32,
}

/// A canvas of this crate as the core's.
pub struct Adapt<'a>(pub &'a mut dyn Canvas);

pub enum AdaptError {
    Refused(Refused),
    /// A read past the text or the glyph table.
    Invalid,
}

impl From<rc2k_core::Invalid> for AdaptError {
    fn from(_: rc2k_core::Invalid) -> Self {
        AdaptError::Invalid
    }
}

impl rc2k_core::text::Canvas for Adapt<'_> {
    type Error = AdaptError;
    fn kerning(&mut self) -> &mut State {
        self.0.kerning()
    }
    fn blit(&mut self, font: &Font, glyph: u32, x: i32, y: i32) -> Result<(), AdaptError> {
        let call = BlitCall::new(glyph, x, y, font.palette, font.glyph_table);
        { self.0.blit_glyph(call) }.map_err(AdaptError::Refused)
    }
}

/// A core text function run on a text and a font's glyph table in game memory, with a
/// canvas of this crate. The glyph table is the font's pointer, read inside its allocation
/// (`pointers`): a read past it is refused.
pub fn run_core<T>(
    font: &Font,
    text: *const u8,
    canvas: &mut dyn Canvas,
    f: impl FnOnce(&GameBytes, &Glyphs, &mut Adapt) -> Result<T, AdaptError>,
) -> Result<T, Refused> {
    let glyphs = glyph_store::bytes(font.glyph_table, "a font's glyph table")?;
    let words = GamePtr::new(crate::pointers::game_address(text)).bytes("a text")?;
    match f(&words, &glyphs, &mut Adapt(canvas)) {
        Ok(value) => Ok(value),
        Err(AdaptError::Refused(e)) => Err(e),
        Err(AdaptError::Invalid) => {
            Err(crate::errors::BadInput("a read past a font's glyph table").into())
        }
    }
}

/// The core's outcome with the text as a pointer.
pub fn outcome(text: *const u8, out: rc2k_core::text::Outcome) -> Outcome {
    Outcome {
        too_wide: out.too_wide,
        text: text.wrapping_add(out.next),
        x: out.x,
        y: out.y,
    }
}

/// `font` is changed only in `MODE_MEASURE`. An error can only come from the blitter; by then
/// earlier glyphs are drawn. Converted: `rc2k_core::text::draw`.
pub fn draw_text(
    font: &mut Font,
    text: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let copy = *font;
    let out = {
        run_core(&copy, text, canvas, |words, glyphs, c| {
            rc2k_core::text::draw(font, words, glyphs, x, y, c)
        })
    }?;
    Ok(outcome(text, out))
}

/// `draw_text_at` on a text of Rust's own (a typed table's name held by value; ended by a
/// NUL in it, or by its end, which the core refuses as a text that does not end). Gives
/// the core's outcome: `next` is where the text after it starts, in `text`.
pub fn draw_bytes_at(
    font: *mut Font,
    text: &[u8],
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<rc2k_core::text::Outcome, Refused> {
    let at = GamePtr::new(crate::pointers::game_address(font));
    let before = at.read::<Font>("a font")?;
    let mut copy = before;
    let glyphs = glyph_store::bytes(copy.glyph_table, "a font's glyph table")?;
    let out = match rc2k_core::text::draw(&mut copy, text, &glyphs, x, y, &mut Adapt(canvas)) {
        Ok(out) => out,
        Err(AdaptError::Refused(e)) => return Err(e),
        Err(AdaptError::Invalid) => {
            return Err(
                crate::errors::BadInput("a read past a text or a font's glyph table").into(),
            );
        }
    };
    if copy != before && canvas.keeps_font_changes() {
        at.write(copy, "a font")?;
    }
    Ok(out)
}

/// `draw_text` with a font object in game memory, for Rust callers. The object is only
/// written (it only changes in `MODE_MEASURE`) if the canvas keeps font changes.
pub fn draw_text_at(
    font: *mut Font,
    text: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let before = crate::pointers::GamePtr::new(crate::pointers::game_address(font))
        .read::<Font>("a font")?;
    let mut copy = before;
    let out = draw_text(&mut copy, text, x, y, canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        crate::pointers::GamePtr::new(crate::pointers::game_address(font)).write(copy, "a font")?;
    }
    Ok(out)
}
