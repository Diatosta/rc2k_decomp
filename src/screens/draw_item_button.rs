//! `screens::draw_item_button` (0x469283): draws the button a menu item describes.
//!
//! In:  edi = the menu item
//! Out: every register preserved
//! Width from the item's +8, alignment from +0xC, position from +0 and +4; through
//! `draw_button_highlighted` if the item is the highlighted one (`dword_755525` equals its
//! type at +0x10), else through `draw_button`. Two callers, both in `draw_menu_items`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::buttons::draw_item_button;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version: `buttons::draw_item_button` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_item_button(r.edi, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_item_button", r, &rust)
}
