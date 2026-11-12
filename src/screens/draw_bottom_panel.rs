//! `screens::draw_bottom_panel` (0x49FCCF): a page's backdrop: the whole page above 0x1E3
//! and the bottom bar round its middle, see-through blue 0x11166A (the fade's full see-through), the
//! middle's strip dark grey 0x1F1F1F; each a `scene::draw_tinted_quad`.
//!
//!
//! In a menu loop's draw list (0x7590BA). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

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
