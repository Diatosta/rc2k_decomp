//! `screens::draw_fade_overlay` (0x49FC7F): a see-through quad over the whole 800 x 600
//! screen, colour 0x11166A, its see-through `dword_75AB50` (the fades of the inspection's
//! screens), by `scene::draw_tinted_quad`; the quad's inputs `dword_623E10` ..
//! `dword_623E24` (`screens::service_boxes::quad`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A1E60` (the fades' frame).

use crate::errors::Refused;
use crate::screens::service_boxes::quad;

/// The Rust version (see the header).
pub fn draw_fade_overlay() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::fade_overlay`.
    let a = rc2k_core::screens::panels::fade_overlay(
        crate::menu::owner::menus()
            .screens
            .state
            .menus
            .fade_see
            .get(),
    );
    quad(a.x, a.y, a.w, a.h, a.colour, a.see)
}
