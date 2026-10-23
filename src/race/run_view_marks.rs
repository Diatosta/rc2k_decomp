//! `race::run_view_marks` (0x456336): the drawn view's marks: which show, the layout, the car's turn, the cars shared, the records' first words, the screen centre for the layout around `marks::frame_marks` and `marks::run_frame` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block or car record, or no marks settings. Callers:
//! `sub_4559A6`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_views::run_view_marks;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { run_view_marks(r) }?;
    Ok(())
}
