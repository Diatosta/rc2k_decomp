//! Drawing and measuring text: a text is drawn glyph by glyph in one of the game's fonts, with
//! the font's spacing and kerning, at a position on the current drawing surface; the same code
//! measures a text without drawing it. The fonts and glyphs are the graphics owner's; a font's
//! record is described in `font.rs`.

pub mod blend_font;
pub mod draw_clock_fixed_pitch;
pub mod draw_string_by_id;
pub mod draw_text;
pub mod draw_text_keep_regs;
pub mod draw_time_fixed_pitch;
pub mod draw_time_text;
pub mod font;
pub mod glyph_kerning;
pub mod measure_string_by_id;
pub mod measure_text;
