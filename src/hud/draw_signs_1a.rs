//! `hud::draw_signs_1a` (0x4B52F7): the first player's signs in layout 1: as `hud::draw_signs` from the record's +0xA4 ..,
//! the places from +0xB4 less 0xC8, at row 0x70 (0x20 higher with `dword_6D1502` 0).
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
pub fn draw_signs_1a(record: GamePtr) -> Result<(), Refused> {
    signs_with(record, 0xA4, 0xB4, 0xFFFFFF38, 0x70, 0x20)
}
