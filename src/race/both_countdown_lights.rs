//! `race::both_countdown_lights` (0x45AFDB): `race::countdown_lights` for the first view
//! (edi 0, flag `dword_6D1830`) and the second (edi 0x10, `dword_6D1834`).
//!
//! In:  nothing
//! Out: edi 0x10, esi the second block, ebp 0x6D1834, the rest as the second call leaves
//!      them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB4F), each frame drawn. Shadow
//! mode: by state. Self-test: the game's view blocks' +0x14/+0x1C set for times around the
//! steps; all put back (skipped without view blocks).
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
    Ok(all_regs(&unsafe { both_countdown_lights(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
