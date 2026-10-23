//! `race::scroll_map` (0x45B527): the work copy's car on the course map, its picture's
//! corner moved and redrawn when the car nears an edge, its place in the picture; see
//! `race/map_scroll.rs`.
//!
//! In:  nothing (the work copy `g_car`)
//! Out: the registers as it leaves them (its caller goes on with a call)
//!
//! Refuses (before anything) no course faces or points. Callers: `race::scroll_maps`
//! (three). Shadow mode runs the original alone (the redraw writes a picture). Self-test: a
//! copy whose redraw goes to a stand-in, against the Rust version with the same: a test
//! face and point, places inside and outside the picture, fewer than two checkpoints.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::map_scroll::scroll_part;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    scroll_part(1, r)?;
    Ok(())
}
