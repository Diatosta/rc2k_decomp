//! `race::credit_line` (0x4161BA): string eax in the font at ebx at (ecx, edx), unless
//! edx is at or below -0x1B or at or past the screen's bottom (see `race/frame_text.rs`).
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y
//! Out: as `text::draw_string_by_id` leaves them, or all kept
//!
//! Callers: `race::roll_credits` (eight places). Shadow mode: the drawing harness.
//! Self-test: a string in the credits' font at heights above, on and below the screen
//! (skipped without the blitter's recording or the font).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_text::credit_line_at;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { credit_line_at(r, canvas) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("race::credit_line", r, &rust)
}
