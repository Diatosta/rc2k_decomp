//! Drawing and measuring text: a text is drawn glyph by glyph in one of the game's fonts, with
//! the font's spacing and kerning, at a position on the current drawing surface; the same code
//! measures a text without drawing it. The fonts and glyphs are the graphics owner's; a font's
//! record is described in `font.rs`.

pub(crate) mod blend_font;
pub(crate) mod draw_clock_fixed_pitch;
pub(crate) mod draw_string_by_id;
pub(crate) mod draw_text;
pub(crate) mod draw_text_keep_regs;
pub(crate) mod draw_time_fixed_pitch;
pub(crate) mod draw_time_text;
mod font;
pub(crate) mod glyph_kerning;
pub(crate) mod measure_string_by_id;
pub(crate) mod measure_text;
