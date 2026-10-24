//! `screens::draw_rally_times_heading` (0x48AAF6): a rally stage times page's heading: the stage's name and its stage line (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `screens::draw_rally_stage_times`, 0x489571, 0x489AE3. Refuses (before anything) no stage records. Compared in shadow mode
//! (`dispatch_draw`). Self-test: the page's own stage and the rally's stages 0 .. 2, nothing drawn (skipped
//! without the fonts or the records).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::GameCanvas;

/// The Rust version, on the game's screen.
pub unsafe fn draw_rally_times_heading() -> Result<(), Refused> {
    unsafe { draw_times_heading(&RALLY_STAGE_TIMES, &mut GameCanvas) }
}
