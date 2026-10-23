//! `race::draw_list_entry` (0x454FF0): one sort or depth entry drawn by its second word
//! (see `race/frame_draw.rs`): a course face (kind 0xFF by `scene::draw_described_object`,
//! below 0x10 by `race::draw_tinted_face`), a
//! flare (`race::draw_flare`), a car (`race::draw_car_entry`) or a lamp
//! (`race::draw_lamp_flare`); `dword_6365C8` 0 after.
//!
//! In:  ebx = the word
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `race::draw_sorted_entries`, `race::draw_depth_entries`. Refuses (before
//! anything) a word of 0x4000000 or more. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose five drawing calls go to stand-ins that note eax .. ecx,
//! against the Rust version on the same; test faces (`g_courseFaces` pointed at them),
//! each kind of word; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_list_entry(r, &mut entry_part) }?;
    Ok(())
}
