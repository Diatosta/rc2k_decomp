//! `race::view_matrices` (0x4386A4): the view's matrices for the frame (see
//! `race/frame_draw.rs`): the turn (kept with `dword_63993C`; the car's undone for its
//! views 0 .. 3; else the identity turned twice by the view's six angles), views 1 and 2
//! rolled with the car (eased into `g_car` +0x6A0 by its speed), then the place:
//! `g_placedMatrix` (kept at 0x5EA1CC), `g_viewMatrix`, `g_viewBackMatrix`. With
//! `scene::matrix` / `scene::view_matrix`'s Rust versions; x87 as the original.
//!
//! In:  nothing
//! Out: eax and ebx the place's x and y negated, ecx 0, edx 0, esi `g_viewTurnMatrix`, edi
//!      `g_viewBackMatrix`, ebp `g_viewMatrix` (a `retn` follows)
//!
//! Caller: `race::camera_view` (0x438467). Shadow mode: by state (the matrices, their
//! work dwords, the inversion's work, `g_car` +0x6A0, `dword_62400C`). Self-test: the
//! angles, place, car's turn, speed and roll set by pattern, each view kind, the kept
//! turn; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { view_matrices(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
