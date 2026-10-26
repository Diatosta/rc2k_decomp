//! `race::lay_skidmarks_if_on` (0x4585F3): `race::lay_skidmarks` unless `dword_6235E8` or
//! `dword_6416BC` is set (see `race/skid_laying.rs`).
//!
//! In:  nothing
//! Out: as `lay_skidmarks` leaves them, or all kept
//!
//! Caller: `sub_45429B` (0x454795). Shadow mode runs the original alone. Self-test: a copy
//! whose call goes to a stand-in, the switches each way.

use crate::framework::Regs;
use crate::race::skid_laying::{lay_part, lay_skidmarks_if_on};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { lay_skidmarks_if_on(r, &mut lay_part) }?;
    Ok(())
}
