//! `screens::learn_axis` (0x46B131): the controller page's learning of a control's axis,
//! direction or button for up to 3 s (see `screens/controller.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In four controller pages' draw lists (0x758D96 ..). Refuses (before anything) no current
//! player record.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::screens::controller::learn_axis;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::learn_axis") {
        return Ok(());
    }
    unsafe { learn_axis(r) }?;
    Ok(())
}
