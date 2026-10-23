//! `race::draw_time_plate` (0x45B70A): slot eax's plate below the name: cleared, the two time texts drawn (see `race/name_plates.rs`).
//!
//! In:  eax = the slot
//! Out: every register preserved
//!
//! It begins with its lock's call: private copy (0xA7, its calls relocated), manual
//! patch. The surface's lock and unlock are the platform's (section 15); the text is
//! `text::draw_text_keep_regs`. Callers: `race::update_name_plates`. Shadow mode runs the original alone (it
//! writes a surface). Self-test: a copy whose lock, unlock and text calls go to stand-ins
//! (`race/plate_test.rs`: a test surface), against the Rust version with the same: line
//! heights, a failed lock.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::name_plates::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_time_plate(r, &mut plate_part) }?;
    Ok(())
}
