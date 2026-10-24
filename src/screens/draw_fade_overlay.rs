//! `screens::draw_fade_overlay` (0x49FC7F): a see-through quad over the whole 800 x 600
//! screen, colour 0x11166A, its see-through `dword_75AB50` (the fades of the inspection's
//! screens), by `scene::draw_tinted_quad`; the quad's inputs `dword_623E10` ..
//! `dword_623E24` (`screens::service_boxes::quad`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A1E60` (the fades' frame). The quad through `part` (1), in the game its
//! Rust version. Shadow mode runs the original alone (it draws). Self-test:
//! `service_boxes::test_boxes`, the see-through by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::screens::service_boxes::{quad, rd};

const SEE_THROUGH: usize = 0x75AB50; // dword_75AB50

/// The Rust version (see the header).
pub fn draw_fade_overlay() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::fade_overlay`.
    let a = rc2k_core::screens::panels::fade_overlay(rd(SEE_THROUGH));
    quad(a.x, a.y, a.w, a.h, a.colour, a.see)
}
