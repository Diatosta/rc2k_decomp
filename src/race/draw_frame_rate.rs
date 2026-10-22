//! `race::draw_frame_rate` (0x4562B2): the frame rate ("fps: n", `flt_6214F0` rounded) at
//! (0x190, 4) in the font at 0x51DE90 when it is not 0 and `dword_5EB100` is set;
//! `dword_5C6604` 2 (see `race/frame_text.rs`).
//!
//! In:  nothing
//! Out: as `text::draw_text` leaves them, or all kept (a `call` follows)
//!
//! Caller: `sub_4547AE` (0x4547D2), each frame.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_text::draw_frame_rate;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_frame_rate(canvas) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("race::draw_frame_rate", r, &rust)
}
