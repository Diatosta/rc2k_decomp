//! `hud::draw_signs_1b` (0x4B5451): the second player's signs in layout 1: as `hud::draw_signs` from the record's +0xB8 ..,
//! the places from +0xC8 plus 0xC8, at row 0x70 (0x20 higher with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_signs`'
//!
//! Caller: layout 1's drawer.

use crate::errors::Refused;
use crate::hud::draw_signs::signs_with;
use crate::pointers::GamePtr;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub fn draw_signs_1b(record: GamePtr) -> Result<(), Refused> {
    signs_with(record, 0xB8, 0xC8, 0xC8, 0x70, 0x20)
}
