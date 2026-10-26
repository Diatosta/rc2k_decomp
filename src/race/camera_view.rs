//! `race::camera_view` (0x438325): the view for the frame (see `race/frame_draw.rs`):
//! `dword_606274`, `dword_60626C`, `dword_606270` from the work car; without
//! `dword_621480` `race::place_view`; else the place and angles from the car's camera
//! (or, with `dword_63993C`, from the crash: the kept turn made, the place the car's raised
//! by 0x8000), as whole numbers and floats (also `g_car` +0x734 ..) and
//! `scene::set_view_window`; then `race::view_matrices`.
//!
//! In:  nothing
//! Out: as `view_matrices` leaves them (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x4557E5), each view drawn.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { camera_view(r) }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
