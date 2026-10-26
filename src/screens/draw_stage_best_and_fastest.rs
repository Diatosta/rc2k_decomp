//! `screens::draw_stage_best_and_fastest` (0x4744A9): a rally stage's best time (of the times at
//! edi) and fastest speed (of the stage record) lines (see `screens/stage_info.rs`).
//!
//! In:  edi = the stage's times (0x9C-byte entries)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller `screens::draw_rally_stage_info`. Refuses (before anything) no stage record.
//! Compared in shadow mode (`dispatch_draw`). Self-test: the rally's stages 0 .. 2, nothing drawn (skipped
//! without the fonts or the records).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::screens::stage_info::*;
use crate::text::draw_text::Canvas;

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let _ = r;
    unsafe { draw_stage_best_and_fastest(r.edi, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_stage_best_and_fastest", r, &rust)
}
