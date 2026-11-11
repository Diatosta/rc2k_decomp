//! `screens::draw_column_boxes` (0x4A0078): a two-column page's 32 boxes: in each column (x
//! 0x43, then 0x192; rows from y 0x45 every 0x1D, numbered by `draw_work.figure` 1 .. 0x20, the
//! place in `g_markerX` / `g_markerY`) a box 0x14D x 0x1B (0x4FCF, see-through
//! `state.service_ui`) and its black border; the chosen row (`loop_flags.item`, while
//! `draw_work.blink` is set) also a yellow (0xFFFF00) border and one a pixel outside it. Each a
//! `scene::draw_tinted_quad`.
//!
//!
//! Caller: `screens::draw_two_column_panel`.

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::screens::service_boxes::{quad, rd, set};
use crate::session::Dword;

const ROW: &Dword = &crate::menu::owner::MENUS.screens.draw_work.figure;
const MARKER_X: &Dword = &crate::menu::owner::MENUS.screens.draw_work.marker_x; // g_markerX
const MARKER_Y: &Dword = &crate::menu::owner::MENUS.screens.draw_work.marker_y; // g_markerY

pub fn draw_column_boxes(screens: &Screens) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::panels::row_box`; the row and marker variables
    // are left as the original leaves them.
    use rc2k_core::screens::panels::{BOXES_Y, COLUMNS, ROW_STEP, ROWS, row_box};
    set(ROW, 1);
    for x in COLUMNS {
        set(MARKER_X, x);
        set(MARKER_Y, BOXES_Y);
        for _ in 0..ROWS {
            let chosen =
                screens.loop_flags.item.get() == rd(ROW) && screens.draw_work.blink.is_set();
            for a in row_box(
                rd(MARKER_X),
                rd(MARKER_Y),
                screens.state.service_ui.box_colour_a.get(),
                chosen,
            ) {
                quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
            }
            set(MARKER_Y, rd(MARKER_Y).wrapping_add(ROW_STEP));
            set(ROW, rd(ROW).wrapping_add(1));
        }
    }
    Ok(())
}
