//! `race::draw_lamp_flare` (0x4551F4): lamp ebx (`g_lamps`) as a flare: its point, size
//! (+4 x 8) and colour (+8), object `dword_621508`, placed by `race::place_object_matrix`
//! and drawn (`scene::draw_scene_object`), see `race/frame_draw.rs`.
//!
//! In:  ebx = the lamp
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `race::draw_list_entry` (lamps 0x3000000 ..). Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose drawing call goes to a stand-in, against the
//! Rust version on the same; test lamps on points of the test's; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_lamp_flare(r, &mut entry_part) }?;
    Ok(())
}
