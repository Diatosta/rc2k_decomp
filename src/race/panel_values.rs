//! `race::panel_values` (0x45AB5A): both view blocks' values into the panel fields at
//! 0x6D13FE .. 0x6D1472: +0x718 rounded (`fistp`), +0xAB0, +0x1D4, the distance from
//! checkpoint `dword_60102C` to the block's +0x98 (`g_checkpointDistances`, halved, then
//! twice that over 65536.0 and over `g_metresInMile`, a float), +0x88 x 10, +0xA0C,
//! -(+0xA10), +0xA14; for the first view also +0xA18, +0xA1C, +0xA20, +0xA24.
//!
//! In:  nothing
//! Out: esi the second view's block, ebx `dword_60102C`, eax its +0x88 x 10 (a `call`
//!      follows); `dword_62400C` (a scratch dword) the second view's halved distance
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB36), each frame drawn. The
//! fields' meaning (speed, gear, distance in miles, time: a reading) is not settled. The
//! checkpoint indices are not checked (the original reads the table unchecked). The Rust
//! version is `panel_values_plain` (`set_up_panel` keeps the exact one). Shadow mode: by
//! state, the two distances within `FLOAT`. Self-test, at the test precisions: the game's
//! view blocks with patterns, checkpoint distances 0 .. 7 set, both orders of distance;
//! all put back (skipped without view blocks).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

// g_checkpointDistances
// dword_60102C

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_values_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
