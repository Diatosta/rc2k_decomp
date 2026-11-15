//! Updates the kerning row table for one glyph and returns how far the glyph can move left (the
//! amount to subtract from x).

/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;
