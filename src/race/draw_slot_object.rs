//! `race::draw_slot_object` (0x4626D1): a race slot's object drawn (see
//! `race/frame_draw.rs`): its number to `g_objectNumber`; outside one player and modes 0
//! and 2 only the drawn view's car; the light set, the slot's turn and place (+0x10, +4 ..
//! with +0x5C on y) moved by the view into `g_madeObjectMatrix`, then
//! `scene::draw_scene_object` with the reflections kept around it.
//!
//! In:  esi = the slot (0x78 bytes of `g_raceSlots`)
//! Out: the registers as the drawing leaves them (its caller calls it inside `pusha`)
//!
//! In the game the drawing is `scene::draw_scene_object_in_game`. Refuses (before
//! anything) a car's object without its block. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose drawing call goes to a stand-in that notes eax .. ecx,
//! against the Rust version on the same; test slots (objects, places, turns), players,
//! modes and views; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_slot_object(r) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
