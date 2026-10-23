//! `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face has a
//! check (+0x26 not 0), up to `g_lastCheckpoint` (0x623644, not included).
//!
//! In:  nothing
//! Out: eax = that checkpoint, 0 for none or with `g_checkpointCount` - 2 negative (then
//!      nothing else changed); else edx = the checkpoint (`g_lastCheckpoint` for none),
//!      ebx and esi = the last face looked at (number, address); the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4464AD` (the camera's start, under
//! `sub_446463`, race init's first step; not with `dword_62389C` 4). Refuses a look that
//! would run past the checkpoint list (`g_lastCheckpoint` 0 or 1 with no check found: the
//! original runs on). Shadow mode: the Rust version and the original, registers compared
//! (it writes nothing). Self-test: faces and checkpoints of the test's (`TestLine`), the
//! game's put back; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, ESI, Regs};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { first_check_checkpoint(r) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EDX, g.edx)
        .with(EBX, g.ebx)
        .with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
