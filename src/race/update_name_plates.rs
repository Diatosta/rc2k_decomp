//! `race::update_name_plates` (0x45B3B9): the time plate drawn again when its texts
//! changed, and each view's asked-for name drawn on its plate (by string id or text); see
//! `race/name_plates.rs`.
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block. Caller: `sub_4547AE` (0x45479A).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::name_plates::update_name_plates;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { update_name_plates(r) }?;
    Ok(())
}
