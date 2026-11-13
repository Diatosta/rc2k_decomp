//! A page's backdrop: the whole page above the bottom bar and the bar round its middle in
//! see-through blue, the middle's strip dark grey, each a `scene::draw_tinted_quad`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

pub fn draw_bottom_panel(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::panels::bottom_panel`.
    for a in
        rc2k_core::screens::panels::bottom_panel(menus.screens.state.service_ui.fade_full.get())
    {
        quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
