//! `screens::draw_overall_best_scores` (0x4A8FA2): the overall best scores (group 4) with its heading
//! (the section's name in event mode 0x13, else string 0x362), the player's own scores in
//! the highlight fonts (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D42). Refuses (before anything) no table. Hook removed
//! 2026-10-22 (typed tables): the menus and draw lists run its `entry` (`menu::table_calls`);
//! the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::best_scores::*;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_scores_with_heading(&OVERALL_BEST_SCORES, false, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_overall_best_scores", r, &rust)
}
