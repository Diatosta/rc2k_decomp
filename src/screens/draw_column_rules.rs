//! The see-through blue rules between a two-column page's sixteen rows (`scene::draw_tinted_quad`).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

pub(super) fn draw_column_rules(o: Owners<'static>) -> Result<(), Refused> {
    let screens = &o.menus.screens;
    // The logic is in `rc2k_core::screens::panels::column_rules`.
    for a in rc2k_core::screens::panels::column_rules(screens.state.read().service_ui.fade_full) {
        quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
