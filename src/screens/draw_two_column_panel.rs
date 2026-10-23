//! `screens::draw_two_column_panel` (0x49FE4B): a two-column page's backdrop: the top bar,
//! the two side bars, the bottom bar and the middle rule (see-through blue 0x11166A,
//! `dword_75AA94`; `scene::draw_tinted_quad`), then the rows' rules
//! (`screens::draw_column_rules`) and boxes (`screens::draw_column_boxes`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In two menu loops' draw lists (0x758F12, 0x758F1A).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::service_boxes::{Part, quad, quad_part, rd};

/// The Rust version: every register kept. Parts: 1 the quads, 2 the rules, 3 the boxes.
pub unsafe fn draw_two_column_panel(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::screens::panels::two_column_panel`.
    let mut part = part;
    let mut g = *entry;
    for a in rc2k_core::screens::panels::two_column_panel(rd(0x75AA94)) {
        quad(&mut g, a.x, a.y, a.w, a.h, a.colour, a.see, &mut part)?;
    }
    part(2, &mut g)?;
    part(3, &mut g)?;
    Ok(*entry)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::screens::{draw_column_boxes, draw_column_rules};
    unsafe {
        match place {
            1 => quad_part(r),
            2 => {
                *r = draw_column_rules::draw_column_rules(r, &mut draw_column_rules::game_part)?;
                Ok(())
            }
            _ => {
                *r = draw_column_boxes::draw_column_boxes(r, &mut draw_column_boxes::game_part)?;
                Ok(())
            }
        }
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_two_column_panel(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
