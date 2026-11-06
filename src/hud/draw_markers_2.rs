//! `hud::draw_markers_2` (0x4B55EE): `hud::draw_markers_1`'s twin at rows 0x11F and
//! 0x12C (the lower half's).
//!
//! In:  ebp the HUD's record
//! Out: as `draw_markers_1`'s
//!
//! Caller: layout 2's drawer (0x4B403B).

use crate::errors::Refused;
use crate::hud::draw_markers_1::markers_with;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}` (through `markers_with`)
pub fn draw_markers_2(frame: &Frame, hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    markers_with(frame, hud, record, [0x11F, 0x12C])
}
