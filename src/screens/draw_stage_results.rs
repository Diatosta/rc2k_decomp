//! `screens::draw_stage_results` (0x48E740): a time trial stage's results: the menu's items,
//! the stage line, ten rows of name, time (with the gap to the first) or seal code, rank, car
//! and flag, and the fastest line (see `screens/stage_results.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D06). Refuses (before anything) no current menu, stage
//! records or times table. Hook removed 2026-10-22 (typed tables): the menus and draw lists run
//! its `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::screens::stage_results::draw_stage_results;
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_stage_results(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_stage_results", r, &rust)
}
