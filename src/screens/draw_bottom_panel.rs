//! `screens::draw_bottom_panel` (0x49FCCF): a page's backdrop: the whole page above 0x1E3
//! and the bottom bar round its middle, see-through blue 0x11166A (`dword_75AA94`), the
//! middle's strip dark grey 0x1F1F1F; each a `scene::draw_tinted_quad`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x7590BA). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::screens::service_boxes::{quad, rd};

/// The Rust version: every register kept.
pub unsafe fn draw_bottom_panel() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::bottom_panel`.
    for a in rc2k_core::screens::panels::bottom_panel(rd(0x75AA94)) {
        quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
