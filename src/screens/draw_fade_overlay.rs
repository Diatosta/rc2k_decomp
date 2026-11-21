//! A see-through quad over the whole 800 x 600 screen, its see-through the fade's
//! (`scene::draw_tinted_quad`).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

pub fn draw_fade_overlay(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::panels::fade_overlay`.
    let a = rc2k_core::screens::panels::fade_overlay(menus.screens.state.read().menus.fade_see);
    quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)
}
