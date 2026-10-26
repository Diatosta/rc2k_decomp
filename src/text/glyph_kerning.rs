//! `text::glyph_kerning` (0x402577): updates the kerning row table for one glyph and returns
//! how far the glyph can move left.
//!
//! In:  eax = glyph index, esi = glyph table
//! Out: eax = kerning (amount to subtract from x); everything else preserved
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::{game_ptr, rva};

// ---- game data ----

const STATE_RVA: usize = rva(0x93E830); // g_textLineState: the kerning row table
pub const STATE_LEN: usize = rc2k_core::text::KERNING_ROWS * 2;
/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;

pub fn state_ptr() -> *mut State {
    game_ptr(STATE_RVA)
}

// ---- Rust version ----
