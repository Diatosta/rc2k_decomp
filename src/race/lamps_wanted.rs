//! `race::lamps_wanted` (0x4536D7): `dword_63DB58` = `g_lampCount`, with bit 0 set when a
//! race car has +0x25C and +0x84 both set.
//!
//! In:  nothing
//! Out: eax the value, ecx the cars (0 with none), esi the last car's block (when any);
//!      the rest kept (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455826), each view drawn. What +0x25C
//! and +0x84 are (the lights on, the car running: a reading) is not settled. Refuses
//! (before anything) more than eight cars.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lamps_wanted() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
