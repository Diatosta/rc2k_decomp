//! `race::draw_race_slots` (0x4621E5): `race::draw_slot_object` for each of the eight race
//! slots (0x78 bytes at `g_raceSlots`) in use (+0 not 0), inside `pusha`.
//!
//! In:  nothing
//! Out: ecx 8, esi past the slots; the rest kept (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x45583F), each view drawn. Refuses
//! (before anything) a slot's car without its block.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe { draw_race_slots(r, &mut |s| draw_slot_object(s).map(|_| ())) };
    {
        let after = done?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
