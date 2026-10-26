//! `screens::draw_class_times_marked` (0x48AB87): the class `dword_7587EC`'s times, the
//! player's own (+0x40 1) in the highlight fonts, the sprites 4 lower (see
//! `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D32). Refuses (before anything) no table. Hook removed
//! 2026-10-22 (typed tables): the menus and draw lists run its `entry` (`menu::table_calls`);
//! the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::best_scores::*;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_scores(&MARKED_CLASS_TIMES, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_class_times_marked", r, &rust)
}
