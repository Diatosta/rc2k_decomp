//! `race::draw_own_car` (0x455AD3): the drawn view's car seen from inside drawn (depth cleared first when asked), then the marks' overlays (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) the car without its block. Callers: `sub_4559A6`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_views::{draw_own_car, own_car_part};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_own_car(r, &mut own_car_part) }?;
    Ok(())
}
