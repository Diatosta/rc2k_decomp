//! `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0 after the drawing (see
//! `race::start_kept_reflections`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Callers: `race::draw_slot_object`, `sub_4616AA` (0x461DFC). Shadow mode: by state.
//! Self-test: the switch 1 and 0xAAAAAAAA first; put back.
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
    unsafe { end_kept_reflections() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
