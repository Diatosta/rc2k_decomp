//! `text::glyph_kerning` (0x402577): updates the kerning row table for one glyph and returns
//! how far the glyph can move left.
//!
//! In:  eax = glyph index, esi = glyph table
//! Out: eax = kerning (amount to subtract from x); everything else preserved
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::moved::Place;

// ---- game data ----

/// g_textLineState (0x93E830 in the image, bss): the kerning row table, out of the image since
/// 2026-10-24 (phase 5); 0 at the start.
pub static STATE: Place<State> = Place::new([0; rc2k_core::text::KERNING_ROWS * 2]);
/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;

pub fn state_ptr() -> *mut State {
    STATE.ptr()
}

// ---- Rust version ----
