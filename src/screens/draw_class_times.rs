//! `screens::draw_class_times` (0x46D162): a group of the class times, ten rows of name, time, rank, car and flag (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In `menu::view_best_scores_loop`'s draw list, through `screens::draw_best_scores_page`. Refuses (before anything) no table.
//! Compared in shadow mode (`dispatch_draw`). Self-test: page 5 and others above 4, nothing drawn (skipped
//! without the fonts or the tables).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::screens::best_scores::{CLASS_TIMES, draw_scores};
use crate::text::draw_text::GameCanvas;

/// The Rust version, on the game's screen.
pub unsafe fn draw_class_times() -> Result<(), Refused> {
    unsafe { draw_scores(&CLASS_TIMES, &mut GameCanvas) }
}
