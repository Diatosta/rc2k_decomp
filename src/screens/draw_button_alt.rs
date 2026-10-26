//! `screens::draw_button_alt` (0x468EDD): draws a menu button with its plain end pieces, like `draw_button`
//! without its slips and with work variables of its own.
//!
//! In:  eax = width of the middle, ebx = x, ecx = y, edx = alignment (1 centre, 2 left,
//!      3 right), edi = the menu item, read for its sprites (+0x20)
//! Out: every register preserved
//! One caller: `draw_item_button_right`. Fatal errors 0x80702023 and 0x80701010.
//! The drawing itself, the alignments and the two fatal errors are in `screens/buttons.rs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::buttons::{ALT, draw_button};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version: `buttons::draw_button` with `ALT` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_button(
            &ALT,
            r.eax,
            r.ebx as i32,
            r.ecx as i32,
            r.edx,
            r.edi,
            canvas,
        )
    }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_button_alt", r, &rust)
}
