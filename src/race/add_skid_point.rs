//! `race::add_skid_point` (0x45860C): a wheel's next skid point across its place at the
//! angle from its last, linked into the strip, the last turned to match (see
//! `race/skid_laying.rs`).
//!
//! In:  esi = the wheel's slot (its last point; -1 none), `dword_643AD8` .. its place
//! Out: edx the new point, ebp the last, esi / eax / ebx / ecx as it leaves them (inside its
//!      caller's `pusha`)
//!
//! Callers: `race::lay_skidmarks` (four). Shadow mode: by state. Self-test: test strips in
//! the game's ring (put back): no last point, a last that starts its strip, one inside a
//! strip, the ring wrapping, a reused point with links.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::all_regs;
use crate::race::skid_laying::add_skid_point;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::skids::add_skid_point`.
    Ok(all_regs(&unsafe { add_skid_point(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
