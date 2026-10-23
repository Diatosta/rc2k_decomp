//! `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (not
//! 0x6D14B2) 0; for each view block, `dword_6D14B2` / `dword_6D14C6` = 400 + (+0xFA0),
//! and its first +0xF98 (at most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6.
//!
//! In:  nothing
//! Out: esi the second block, ebp its count (held to 4), ecx the last count a loop ran
//!      with, eax the last value written (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB4A), each frame drawn. The values
//! (marks on the panel's bar: a reading) are not settled. Shadow mode: by state. Self-test:
//! the game's view blocks, counts 0, 1, 4, 5, 0xFFFFFFFF; all put back (skipped without
//! view blocks).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_marks() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
