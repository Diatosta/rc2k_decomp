//! `screens::draw_two_column_panel` (0x49FE4B): a two-column page's backdrop: the top bar,
//! the two side bars, the bottom bar and the middle rule (see-through blue 0x11166A,
//! `dword_75AA94`; `scene::draw_tinted_quad`), then the rows' rules
//! (`screens::draw_column_rules`) and boxes (`screens::draw_column_boxes`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In two menu loops' draw lists (0x758F12, 0x758F1A). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::screens::service_boxes::quad;

/// The Rust version: every register kept. Parts: 1 the quads, 2 the rules, 3 the boxes.
pub fn draw_two_column_panel() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::two_column_panel`.
    for a in rc2k_core::screens::panels::two_column_panel(
        menus().screens.state.service_ui.fade_full.get(),
    ) {
        quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    crate::screens::draw_column_rules::draw_column_rules()?;
    crate::screens::draw_column_boxes::draw_column_boxes()?;
    Ok(())
}
