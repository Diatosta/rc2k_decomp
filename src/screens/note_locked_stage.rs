//! `screens::note_locked_stage` (0x48ADCE): the message box for a chosen stage that is not open (`byte_758800`), unless cheat 0 is on (see `screens/locked_notes.rs`).
//!
//! In:  nothing (ecx passed on to the message box)
//! Out: eax as it leaves it; the rest kept (the box is `pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x759172).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::screens::locked_notes::{box_part, note_locked_stage};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::note_locked_stage") {
        return Ok(());
    }
    let mut g = *r;
    let left = unsafe { note_locked_stage(&mut g, &mut box_part) }?;
    left.apply(r);
    Ok(())
}
