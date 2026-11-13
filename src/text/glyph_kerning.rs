//! Updates the kerning row table for one glyph and returns how far the glyph can move left (the
//! amount to subtract from x).

/// The kerning row table: 0 at the start.
pub static STATE: std::sync::Mutex<State> =
    std::sync::Mutex::new([0; rc2k_core::text::KERNING_ROWS * 2]);
/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;
