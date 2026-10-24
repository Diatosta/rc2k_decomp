//! `screens::draw_record_heading` (0x4A247D): the record pages' heading (the section's name or its first record's, and the record's stage line) (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers 0x4A2112, 0x4A2F47, `screens::draw_record_times`. Compared in shadow mode (`dispatch_draw`). Self-test: the records 0 .. 2 of the
//! page's section, nothing drawn (skipped without the fonts or the records).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::text::draw_text::GameCanvas;

/// The Rust version (`best_scores::draw_record_heading`), on the game's screen.
pub unsafe fn draw_record_heading() -> Result<(), Refused> {
    unsafe { crate::screens::best_scores::draw_record_heading(&mut GameCanvas) }
}
