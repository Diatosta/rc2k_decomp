//! `screens::draw_column_rules` (0x49FFC5): the rules between a two-column page's sixteen
//! rows: in each column (x 0x43 and 0x192) from y 0x5F every 0x1D a line 0x14C wide and 3
//! high, see-through blue 0x11166A (the fade's full see-through; `scene::draw_tinted_quad`).
//!
//!
//! Caller: `screens::draw_two_column_panel`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

pub fn draw_column_rules(o: Owners<'_>) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    // The logic is in `rc2k_core::screens::panels::column_rules`.
    for a in rc2k_core::screens::panels::column_rules(screens.state.service_ui.fade_full.get()) {
        quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
