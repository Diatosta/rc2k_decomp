//! `screens::draw_rally_stage_times` (0x48A948): a rally stage's best times: the heading, then up to ten rows of name, time, rank, car and flag (the sprites 4 lower) (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D22). Refuses (before anything) no stage records or no
//! times table. Hook removed 2026-10-22 (typed tables): the menus and draw lists run its
//! `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::best_scores::*;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_stage_times(&RALLY_STAGE_TIMES, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_rally_stage_times", r, &rust)
}
