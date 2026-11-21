//! A two-column page's 32 boxes: in each column a see-through box with a black border; the chosen
//! row (while `draw_work.blink` is set) also gets a yellow border and one a pixel outside it. Each
//! is a `scene::draw_tinted_quad`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

pub(super) fn draw_column_boxes(o: Owners<'static>) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    // The logic is in `rc2k_core::screens::panels::row_box`.
    use rc2k_core::screens::panels::{BOXES_Y, COLUMNS, ROW_STEP, ROWS, row_box};
    let mut row = 1u32;
    for x in COLUMNS {
        let mut y = BOXES_Y;
        for _ in 0..ROWS {
            let chosen = screens.loop_flags.read().item == row && screens.draw_work.read().blink;
            for a in row_box(x, y, screens.state.read().service_ui.box_colour_a, chosen) {
                quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
            }
            y = y.wrapping_add(ROW_STEP);
            row = row.wrapping_add(1);
        }
    }
    Ok(())
}
