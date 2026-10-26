//! `race::ready_car_parts` (0x460D5C): each race car with its parts shown: its parts'
//! matrices made (`race::car_part_matrices`) and kept in its block (see
//! `race/car_parts.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) no race cars (the original's loop would run away), more than
//! eight, or one without its block. Caller: `sub_45429B` (0x454429).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::car_parts::{parts_part, ready_car_parts};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { ready_car_parts(r, &mut parts_part) }?;
    Ok(())
}
