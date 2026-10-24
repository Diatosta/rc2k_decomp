//! `screens::draw_column_rules` (0x49FFC5): the rules between a two-column page's sixteen
//! rows: in each column (x 0x43 and 0x192) from y 0x5F every 0x1D a line 0x14C wide and 3
//! high, see-through blue 0x11166A (`dword_75AA94`; `scene::draw_tinted_quad`).
//!
//! In:  nothing
//! Out: ecx -1, edx past the last row; the rest as the last quad leaves them
//!
//! Caller: `screens::draw_two_column_panel`. Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (4 cases).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::screens::service_boxes::{quad, rd};

/// The Rust version (what it leaves in registers is read by no caller).
pub unsafe fn draw_column_rules() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::column_rules`.
    for a in rc2k_core::screens::panels::column_rules(rd(0x75AA94)) {
        quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
