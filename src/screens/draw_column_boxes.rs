//! `screens::draw_column_boxes` (0x4A0078): a two-column page's 32 boxes: in each column (x
//! 0x43, then 0x192; rows from y 0x45 every 0x1D, numbered by `dword_7559EC` 1 .. 0x20, the
//! place in `g_markerX` / `g_markerY`) a box 0x14D x 0x1B (0x4FCF, see-through
//! `dword_75AA90`) and its black border; the chosen row (`dword_755525`, while
//! `dword_7559A4` is 1) also a yellow (0xFFFF00) border and one a pixel outside it. Each a
//! `scene::draw_tinted_quad`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_two_column_panel`. Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (16 cases).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::screens::service_boxes::{quad, rd, set};

const ROW: usize = 0x7559EC; // dword_7559EC
const MARKER_X: usize = 0x755A18; // g_markerX
const MARKER_Y: usize = 0x755A1C; // g_markerY

/// The Rust version: every register kept.
pub unsafe fn draw_column_boxes() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::row_box`; the row and marker variables
    // are left as the original leaves them.
    use rc2k_core::screens::panels::{BOXES_Y, COLUMNS, ROW_STEP, ROWS, row_box};
    set(ROW, 1);
    for x in COLUMNS {
        set(MARKER_X, x);
        set(MARKER_Y, BOXES_Y);
        for _ in 0..ROWS {
            let chosen = rd(0x755525) == rd(ROW) && rd(0x7559A4) == 1;
            for a in row_box(rd(MARKER_X), rd(MARKER_Y), rd(0x75AA90), chosen) {
                quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
            }
            set(MARKER_Y, rd(MARKER_Y).wrapping_add(ROW_STEP));
            set(ROW, rd(ROW).wrapping_add(1));
        }
    }
    Ok(())
}
