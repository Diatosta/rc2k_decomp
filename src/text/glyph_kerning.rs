//! `text::glyph_kerning` (0x402577): updates the kerning row table for one glyph and returns
//! how far the glyph can move left.
//!
//! Takes: glyph index, glyph table
//! Returns: kerning (amount to subtract from x)

// ---- game data ----

/// g_textLineState (0x93E830 in the image, bss): the kerning row table; 0 at the start.
pub static STATE: std::sync::Mutex<State> =
    std::sync::Mutex::new([0; rc2k_core::text::KERNING_ROWS * 2]);
/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;
