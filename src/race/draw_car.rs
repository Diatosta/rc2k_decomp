//! `race::draw_car` (0x4616AA): car esi drawn: marks, matrices, the model's parts' bits and
//! the shown mask, the light, the body and nearer its four parts (each
//! `scene::draw_scene_object`); see `race/car_draw.rs`.
//!
//! In:  esi = the car's block (`dword_60627C` its number)
//! Out: the registers as `race::clear_car_marks` and the rest leave them (its caller returns)
//!
//! It begins with its call: private copy (0x7F7, its fifteen calls relocated), manual patch.
//! Refuses (before anything) in a race the drawn car without its block. Shadow mode runs the
//! original alone (it draws). Self-test: a copy whose drawing calls go to a stand-in noting
//! `g_objectNumber` (the rest real), against the Rust version with the same: a copy of car
//! 0's block, its state bits, switches, depths and the drawn view's own car varied; all put
//! back (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_car(r) }?;
    Ok(())
}
