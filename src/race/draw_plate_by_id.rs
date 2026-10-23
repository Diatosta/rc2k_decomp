//! `race::draw_plate_by_id` (0x41B2D4): string id ebx into the text buffer and drawn on slot eax's plate (`race::draw_name_plate`) (see `race/name_plates.rs`).
//!
//! In:  eax = the slot, ebx = the string id
//! Out: every register preserved
//!
//! Callers: `race::update_name_plates` (two). Shadow mode runs the original alone (it
//! writes a surface). Self-test: a copy whose plate call goes to a stand-in (the strings real),
//! against the Rust version with the same: string ids.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::name_plates::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_plate_by_id(r, &mut by_id_part) }?;
    Ok(())
}
