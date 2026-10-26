//! `screens::draw_column_rules` (0x49FFC5): the rules between a two-column page's sixteen
//! rows: in each column (x 0x43 and 0x192) from y 0x5F every 0x1D a line 0x14C wide and 3
//! high, see-through blue 0x11166A (`dword_75AA94`; `scene::draw_tinted_quad`).
//!
//! In:  nothing
//! Out: ecx -1, edx past the last row; the rest as the last quad leaves them
//!
//! Caller: `screens::draw_two_column_panel`. Shadow mode runs the original alone (it draws).
//! Self-test: `service_boxes::test_boxes` (4 cases).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::service_boxes::{Inputs, Part, quad, quad_part, rd};

pub static INPUTS: Inputs = Inputs {
    tested: &[],
    read: &[0x75AA94],
};

/// The Rust version: the registers it leaves.
pub unsafe fn draw_column_rules(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::screens::panels::column_rules`; ecx counts a column's rows
    // down and edx is the row's y (`push ecx; push edx` .. `pop edx; pop ecx` round each
    // quad), then -1 and the y past the last.
    use rc2k_core::screens::panels::{ROW_STEP, ROWS, column_rules};
    let mut part = part;
    let mut g = *entry;
    for column in column_rules(rd(0x75AA94)).chunks(ROWS as usize) {
        for (n, a) in column.iter().enumerate() {
            (g.ecx, g.edx) = (ROWS - 1 - n as u32, a.y);
            quad(&mut g, a.x, a.y, a.w, a.h, a.colour, a.see, &mut part)?;
        }
        g.ecx = u32::MAX;
        g.edx = column[column.len() - 1].y + ROW_STEP;
    }
    Ok(g)
}

pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    quad_part(r)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::draw_column_rules") {
        return Ok(());
    }
    let left = unsafe { draw_column_rules(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
