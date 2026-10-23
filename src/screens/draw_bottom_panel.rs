//! `screens::draw_bottom_panel` (0x49FCCF): a page's backdrop: the whole page above 0x1E3
//! and the bottom bar round its middle, see-through blue 0x11166A (`dword_75AA94`), the
//! middle's strip dark grey 0x1F1F1F; each a `scene::draw_tinted_quad`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x7590BA).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::service_boxes::{Part, quad, quad_part, rd};

/// The Rust version: every register kept.
pub unsafe fn draw_bottom_panel(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::screens::panels::bottom_panel`.
    let mut part = part;
    for a in rc2k_core::screens::panels::bottom_panel(rd(0x75AA94)) {
        // Each inside `pusha` .. `popa`.
        let mut g = *entry;
        quad(&mut g, a.x, a.y, a.w, a.h, a.colour, a.see, &mut part)?;
    }
    Ok(*entry)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_bottom_panel(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
