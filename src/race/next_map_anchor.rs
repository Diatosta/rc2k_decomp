//! `race::next_map_anchor` (0x445FEE): the next checkpoint whose face has a place on the
//! map.
//!
//! In:  ecx = the checkpoint to look from
//! Out: carry clear: ecx the anchor, ebx and esi its face's number and address; carry set:
//!      ecx `g_checkpointCount`, ebx and esi the last face looked at; the rest preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445FB4` (twice, one of them inside itself: the
//! original loops by jumping back to its own start). Refuses more checkpoints than the
//! lists hold. Self-test: faces and checkpoints of the test's, the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, ESI, Regs};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (g, carry) = unsafe { next_map_anchor(r) }?;
    let mut left = Left::carry(carry).with(ECX, g.ecx);
    if g.ebx != r.ebx || g.esi != r.esi {
        left = left.with(EBX, g.ebx).with(ESI, g.esi);
    }
    Ok(left)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
