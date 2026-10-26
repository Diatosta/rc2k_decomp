//! `screens::draw_column_rules` (0x49FFC5): the rules between a two-column page's sixteen
//! rows: in each column (x 0x43 and 0x192) from y 0x5F every 0x1D a line 0x14C wide and 3
//! high, see-through blue 0x11166A (`dword_75AA94`; `scene::draw_tinted_quad`).
//!
//! In:  nothing
//! Out: ecx -1, edx past the last row; the rest as the last quad leaves them
//!
//! Caller: `screens::draw_two_column_panel`.

use crate::errors::Refused;
use crate::menu::state_store::MENU_STATE;
use crate::screens::service_boxes::quad;

/// The Rust version (what it leaves in registers is read by no caller).
pub fn draw_column_rules() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::panels::column_rules`.
    for a in rc2k_core::screens::panels::column_rules(MENU_STATE.service_ui.fade_full.get()) {
        quad(a.x, a.y, a.w, a.h, a.colour, a.see)?;
    }
    Ok(())
}
