//! `text::draw_text` (0x4023D5): draws NUL-terminated text with a bitmap font, line by line.
//!
//! Takes: font, text, x, y
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

use crate::errors::Refused;
use crate::gfx::blit_glyph::{BlitCall, blit_glyph_on_surface};
use crate::gfx::glyph_store;
use crate::gfx::typefaces::FontRef;
use crate::text::font::Font;
use crate::text::glyph_kerning::{STATE, State};
use rc2k_core::strings::bytes::Bytes;

/// What drawing text needs from outside: the kerning row table and the blitter.
pub trait Canvas {
    /// The fonts' stores: the typefaces a font number is resolved against, the palettes and
    /// glyph tables.
    fn fonts(&self) -> &'static crate::gfx::owner::FontStores;
    fn typefaces(&self) -> &'static crate::gfx::typefaces::FontObjects {
        &self.fonts().typefaces
    }
    /// False for a canvas that only predicts: font objects in game memory are then left alone.
    fn keeps_font_changes(&self) -> bool;
    /// The kerning row table the canvas keeps.
    fn kerning(&self) -> State;
    fn set_kerning(&mut self, kerning: State);
    /// The clip rectangle's right edge now (a glyph's call notes it).
    fn clip_right(&self) -> i32;
    fn blit_glyph(&mut self, call: BlitCall) -> Result<(), Refused>;
}

/// The game's own kerning table and surface.
pub struct GameCanvas {
    gfx: &'static crate::gfx::owner::Gfx,
}

impl GameCanvas {
    pub fn new(gfx: &'static crate::gfx::owner::Gfx) -> Self {
        Self { gfx }
    }
}

impl Canvas for GameCanvas {
    fn fonts(&self) -> &'static crate::gfx::owner::FontStores {
        &self.gfx.fonts
    }
    fn keeps_font_changes(&self) -> bool {
        true
    }
    fn kerning(&self) -> State {
        *STATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn set_kerning(&mut self, kerning: State) {
        *STATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = kerning;
    }
    fn clip_right(&self) -> i32 {
        crate::gfx::surface::clip(self.gfx).right
    }
    fn blit_glyph(&mut self, c: BlitCall) -> Result<(), Refused> {
        blit_glyph_on_surface(c.glyph, c.table, c.x, c.y, c.palette, self.gfx)
    }
}

/// How `draw_text` ended. The original also returns 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Outcome {
    /// A line was wider than the box and the font has `FLAG_FAIL_IF_WIDER`: the carry flag.
    pub too_wide: bool,
    /// The x that was passed in.
    pub x: i32,
    /// The y of the last line, font offset included.
    pub y: i32,
}

/// A canvas of this crate as the core's.
pub struct Adapt<'a> {
    canvas: &'a mut dyn Canvas,
    kerning: State,
}

impl<'a> Adapt<'a> {
    pub fn new(canvas: &'a mut dyn Canvas) -> Self {
        let kerning = canvas.kerning();
        Self { canvas, kerning }
    }
}

/// The kerning table goes back when the drawing ends, whichever way: it lasts from text to text.
impl Drop for Adapt<'_> {
    fn drop(&mut self) {
        self.canvas.set_kerning(self.kerning);
    }
}

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
        &mut self.kerning
    }
    fn blit(&mut self, font: &Font, glyph: u32, x: i32, y: i32) -> Result<(), AdaptError> {
        let call = BlitCall::glyph(
            glyph,
            x,
            y,
            font.palette,
            font.glyph_table,
            self.canvas.clip_right(),
        );
        { self.canvas.blit_glyph(call) }.map_err(AdaptError::Refused)
    }
}

/// The core's error as a refusal.
pub fn refusal<T>(result: Result<T, AdaptError>) -> Result<T, Refused> {
    match result {
        Ok(value) => Ok(value),
        Err(AdaptError::Refused(e)) => Err(e),
        Err(AdaptError::Invalid) => {
            Err(crate::errors::BadInput("a read past a text or a font's glyph table").into())
        }
    }
}

/// `font` is changed only in `MODE_MEASURE`. An error can only come from the blitter; by then
/// earlier glyphs are drawn. The text is read up to where the core stops (a text that does
/// not end is refused). Converted: `rc2k_core::text::draw`.
pub fn draw_text(
    font: &mut Font,
    text: &(impl Bytes + ?Sized),
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let glyphs = glyph_store::bytes(canvas.fonts(), font.glyph_table, "a font's glyph table")?;
    let out = refusal(rc2k_core::text::draw(
        font,
        text,
        &glyphs,
        x,
        y,
        &mut Adapt::new(canvas),
    ))?;
    Ok(Outcome {
        too_wide: out.too_wide,
        x: out.x,
        y: out.y,
    })
}

/// `draw_text` with a font object in game memory, for Rust callers. The object is only
/// written (it only changes in `MODE_MEASURE`) if the canvas keeps font changes.
pub fn draw_text_at(
    font: impl Into<FontRef>,
    text: &(impl Bytes + ?Sized),
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    let font = font.into().object(canvas.typefaces());
    let before = font.get();
    let mut copy = before;
    let out = draw_text(&mut copy, text, x, y, canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.set(copy);
    }
    Ok(out)
}
