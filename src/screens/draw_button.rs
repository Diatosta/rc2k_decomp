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
use crate::screens::buttons::PLAIN;
use crate::text::draw_text::GameCanvas;

/// The Rust version: `buttons::draw_button` with `PLAIN` on the game's screen; `width`
/// (eax), `x` (ebx), `y` (ecx), `align` (edx), `item` (edi).
pub unsafe fn draw_button(
    width: u32,
    x: i32,
    y: i32,
    align: u32,
    item: u32,
) -> Result<(), Refused> {
    unsafe {
        crate::screens::buttons::draw_button(&PLAIN, width, x, y, align, item, &mut GameCanvas)
    }
}
