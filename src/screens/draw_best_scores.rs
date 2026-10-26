//! `screens::draw_best_scores` (0x4A9300): a group of the best scores, ten rows of name, score, rank, car and flag (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In `menu::view_best_scores_loop`'s draw list, through `screens::draw_best_scores_page`. Refuses (before anything) no table.
//! Compared in shadow mode (`dispatch_draw`). Self-test: pages 0 .. 4, nothing drawn (skipped
//! without the fonts or the tables).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::best_scores::*;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_scores(&BEST_SCORES, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_best_scores", r, &rust)
}
