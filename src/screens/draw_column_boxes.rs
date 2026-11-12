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
use crate::owners::Owners;
use crate::screens::service_boxes::{quad, rd, set};

pub fn draw_column_boxes(o: Owners<'static>) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    // The logic is in `rc2k_core::screens::panels::row_box`; the row and marker variables
    // are left as the original leaves them.
    use rc2k_core::screens::panels::{BOXES_Y, COLUMNS, ROW_STEP, ROWS, row_box};
    let row = &screens.draw_work.figure;
    let marker_x = &screens.draw_work.marker_x; // g_markerX
    let marker_y = &screens.draw_work.marker_y; // g_markerY
    set(row, 1);
    for x in COLUMNS {
        set(marker_x, x);
        set(marker_y, BOXES_Y);
        for _ in 0..ROWS {
            let chosen =
                screens.loop_flags.item.get() == rd(row) && screens.draw_work.blink.is_set();
            for a in row_box(
                rd(marker_x),
                rd(marker_y),
                screens.state.service_ui.box_colour_a.get(),
                chosen,
            ) {
                quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
            }
            set(marker_y, rd(marker_y).wrapping_add(ROW_STEP));
            set(row, rd(row).wrapping_add(1));
        }
    }
    Ok(())
}
