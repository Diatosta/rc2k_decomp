//! `screens::draw_rally_stage_info` (0x489AC8): a rally stage's information page (heading,
//! the stage panel's body, best and fastest; see `screens/stage_info.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758C6A). Refuses (before anything) no stage record or times
//! table. Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::screens::stage_info::*;
use crate::text::draw_text::Canvas;

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let _ = r;
    unsafe { draw_rally_stage_info(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_rally_stage_info", r, &rust)
}
