//! `screens::draw_item_button_centred_highlighted` (0x46923F): draws a highlighted, centred button as wide as a menu item's text.
//!
//! In:  edi = the menu item; ecx is passed on to the measuring as its x
//! Out: every register preserved
//! The item's string (+0x14) is measured with the item's font at +0x4C
//! (`text::measure_string_by_id`, which leaves the text in the shared text buffer); the
//! button is that wide plus 12, at the item's +0 and +4, through `draw_button_highlighted`.
//! One caller, `draw_menu_items`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::buttons::{ALIGN_CENTRE, HIGHLIGHTED, draw_text_button};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version: `buttons::draw_text_button` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_text_button(&HIGHLIGHTED, ALIGN_CENTRE, r.edi, r.ecx as i32, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_item_button_centred_highlighted", r, &rust)
}
