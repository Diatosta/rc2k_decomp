//! `hud::draw_signs_2a` (0x4B507F): the first player's signs in layout 2: as `hud::draw_signs` from the record's +0xA4 ..,
//! the places from +0xB4, at row 0x8 (no lift).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_signs`'
//!
//! Caller: layout 2's drawer.

use crate::errors::Refused;
use crate::hud::draw_signs::signs_with;
use crate::pointers::GamePtr;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub fn draw_signs_2a(record: GamePtr) -> Result<(), Refused> {
    signs_with(record, 0xA4, 0xB4, 0, 0x8, 0x0)
}
