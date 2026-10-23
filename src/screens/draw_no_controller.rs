//! `screens::draw_no_controller` (0x46AEAA): string 0x56D at (0x1BC, 0xD5) (see
//! `screens/controller.rs`).
//!
//! In:  nothing
//! Out: as `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! Caller: `screens::draw_controller_page`. Compared in shadow mode (`dispatch_draw`).
//! Self-test: once, nothing drawn (skipped without the fonts).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::controller::draw_no_controller;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_no_controller(canvas) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_no_controller", r, &rust)
}
