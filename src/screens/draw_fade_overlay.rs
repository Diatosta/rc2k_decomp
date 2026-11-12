//! `screens::draw_fade_overlay` (0x49FC7F): a see-through quad over the whole 800 x 600
//! screen, colour 0x11166A, its see-through the fade's see-through (the fades of the inspection's
//! screens), by `scene::draw_tinted_quad`; the quad's inputs a scene work cell ..
//! a scene work cell (`screens::service_boxes::quad`).
//!
//! Caller: `menu::fade_frame` (the fades' frame).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::screens::service_boxes::quad;

/// `draw_fade_overlay_in` for the fades' frame, which holds no owners.
pub fn draw_fade_overlay(menus: &Menus) -> Result<(), Refused> {
    let o = crate::owners::boundary();
    debug_assert!(std::ptr::eq(menus, o.menus));
    draw_fade_overlay_in(o)
}

pub fn draw_fade_overlay_in(o: Owners<'_>) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::panels::fade_overlay`.
    let a = rc2k_core::screens::panels::fade_overlay(menus.screens.state.menus.fade_see.get());
    quad(o, a.x, a.y, a.w, a.h, a.colour, a.see)
}
