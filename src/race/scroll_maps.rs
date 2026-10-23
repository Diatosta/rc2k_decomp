//! `race::scroll_maps` (0x45B4E6): `race::scroll_map` on each view's car, loaded into and
//! stored from the work copy; see `race/map_scroll.rs`.
//!
//! In:  nothing
//! Out: as the last store leaves them
//!
//! Refuses (before anything) a view car without its block or no course faces or points.
//! Caller: `sub_45429B` (0x45442E).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::map_scroll::scroll_maps;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { scroll_maps(r) }?;
    Ok(())
}
