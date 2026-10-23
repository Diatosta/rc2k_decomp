//! `screens::draw_button` (0x4688A1): draws a menu button with its plain end pieces.
//!
//! In:  eax = width of the middle, ebx = x, ecx = y, edx = alignment (1 centre, 2 left,
//!      3 right), edi = the menu item, read for its sprites (+0x20)
//! Out: every register preserved
//! Two callers: `draw_item_button` and `draw_item_button_centred`. Right-aligned it has two
//! slips, reproduced (see `Kind::slips`). Fatal errors 0x80702021 and 0x80701010.
//! The drawing itself, the alignments and the two fatal errors are in `screens/buttons.rs`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::buttons::{PLAIN, draw_button};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version: `buttons::draw_button` with `PLAIN` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_button(
            &PLAIN,
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
    draw_entry("screens::draw_button", r, &rust)
}
