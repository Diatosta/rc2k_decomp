//! `race::fetch_view_value` (0x45550E): `dword_6415D8` = the drawn view's block (`off_63DBB0[dword_63DB9C]`) +0xFBC.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45429B` (five) and `sub_455540`. What the value is is not read.
//! Refuses (before anything) a view index past the two (`off_63DBB0` has two).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { fetch_view_value() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
