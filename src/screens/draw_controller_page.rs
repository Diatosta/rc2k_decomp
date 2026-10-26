//! `screens::draw_controller_page` (0x46A454): the player's controller settings: the device,
//! its mode and strength, the sensitivity, two sliders and the stick's place now (see
//! `screens/controller.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Its other chunks are the shared fatal reporter (0x40763C) and its stub (0x4D9450). In four
//! controller pages' draw lists (0x758D8E ..). Refuses (before anything) no current player
//! record, or no input device for a player that has one. Hook removed 2026-10-22 (typed
//! tables): the menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::controller::draw_controller_page;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_controller_page(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_controller_page", r, &rust)
}
