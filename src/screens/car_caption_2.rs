//! `screens::car_caption_2` (0x469873): the car shown's name as the caption; the twin of
//! `screens::car_caption` (0x469A0E: the same bytes but for a `jmp $+2`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens/car_caption.rs`. In three draw lists (0x758C36, 0x758C46, 0x758C56: the car
//! choice). Refuses what that refuses.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::car_caption::car_caption;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_755990
// g_captionText
fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_caption() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
