//! `race::panel_motion` (0x45AE1D): for each view block, +0xAE8 and +0xAEC x 0.5 / 65536
//! and (-(+0x9C0)) >> 14 x pi x 0.5 / 65536 (floats, x87) into the panel (`flt_6D1432`,
//! `flt_6D1436`, `flt_6D143A`; `flt_6D145A`, `flt_6D145E`, `flt_6D1462`); then
//! `dword_62400C` = `g_sceneClock` - `dword_62290C` - 800, 0 when negative.
//!
//! In:  nothing
//! Out: esi the second block, eax that time (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB45), each frame drawn. The values
//! (+0xAE8/+0xAEC speeds, +0x9C0 a heading: a reading) are not settled. Shadow mode: by
//! state, the six floats within `FLOAT`. The Rust version is `panel_motion_plain`.
//! Self-test, at the test precisions: the game's view blocks with patterns, clocks before
//! and after the 800; all put back (skipped without view blocks).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::panel::{motion, heading_turn, heading, shown_for}`.
    Ok(unsafe { panel_motion_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
