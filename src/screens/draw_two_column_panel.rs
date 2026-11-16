//! A two-column page's backdrop: the top bar, the two side bars, the bottom bar and the
//! middle rule (see-through blue, `scene::draw_tinted_quad`), then the rows' rules
//! (`screens::draw_column_rules`) and boxes (`screens::draw_column_boxes`).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

/// Parts: 1 the quads, 2 the rules, 3 the boxes.
pub fn draw_two_column_panel(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::panels::two_column_panel`.
    for a in
        rc2k_core::screens::panels::two_column_panel(menus.screens.state.service_ui.fade_full.get())
    {
        quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    crate::screens::draw_column_rules::draw_column_rules(o)?;
    crate::screens::draw_column_boxes::draw_column_boxes(o)?;
    Ok(())
}
