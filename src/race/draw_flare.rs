//! `race::draw_flare` (0x455189) and the shared test of `race::draw_lamp_flare`: a flare
//! from the flare tables (or a lamp) placed by `race::place_object_matrix` and drawn
//! (`scene::draw_scene_object`), see `race/frame_draw.rs`.
//!
//! In:  ebx = the flare
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `race::draw_list_entry` (flares 0x1000000 ..). Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose drawing call goes to a stand-in that notes
//! eax .. ecx (its matrix call real), against the Rust version on the same; flares of the
//! test's in the tables; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_flare(r, &mut entry_part) }?;
    Ok(())
}
