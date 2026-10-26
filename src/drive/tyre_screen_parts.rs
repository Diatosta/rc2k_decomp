//! `drive::tyre_screen_parts` (0x41C790): the tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `dword_5C2A1C` and screen
//! rectangles in `dword_5C2A0C`, placed from the layout (`TyreChange::parts`, `wheel`,
//! the car's half by `player`). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.

use crate::errors::Refused;
use crate::race::tyre_store::TYRE_CHANGE as TYRE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::Dword;
use rc2k_core::drive::tyre_screen::fixed_parts;

fn set(cells: &[Dword; 4], v: [u32; 4]) {
    for (cell, x) in cells.iter().zip(v) {
        cell.set(x);
    }
}

/// The Rust version: its quads (`drive::tyre_quad`); every register kept.
// Converted: `rc2k_core::drive::tyre_screen::fixed_parts`.
pub fn tyre_screen_parts() -> Result<(), Refused> {
    let parts = std::array::from_fn(|k| TYRE.parts[k].get());
    let wheel = std::array::from_fn(|k| TYRE.wheel[k].get());
    for (uv, rect) in fixed_parts(parts, wheel, TYRE.player.get() != 0) {
        set(&VIEWER_PANEL.texture_rect, uv); // dword_5C2A1C
        set(&VIEWER_PANEL.screen_rect, rect); // dword_5C2A0C
        {
            crate::drive::tyre_quad::tyre_quad(
                VIEWER_PANEL.texture_rect[0].addr() as u32,
                VIEWER_PANEL.screen_rect[0].addr() as u32,
                0xFF_FFFF,
            )
        }?;
    }
    Ok(())
}
