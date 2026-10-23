//! `screens::draw_menu_labels` (0x471EF0): draws the fixed labels of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved; carry clear
//! The menu's first dword points at a list of labels, 0x10 bytes each: x, y, font, string id.
//! The list ends with -1, but the first label is drawn without being checked for it. Each is
//! drawn with `draw_string_by_id`.
//! One caller, the menu's drawing step `sub_4746B7`.
//! Refuses a menu without a label list, and a list without an end in its first 256 labels.
//! Manual patch with a private copy (0x1D bytes, one call relocated): the loop comes back to
//! the fourth byte, inside what the patch overwrites. The first build hooked it without one
//! and the original's second pass through the loop ran into the patch (2026-10-10).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::text::draw_text::Canvas;

const LABEL_LEN: u32 = 0x10;
const END: u32 = u32::MAX;
const LABEL_LIMIT: u32 = 256;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

// ---- Rust version ----

pub unsafe fn draw_menu_labels(menu: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let labels = dword(menu);
        if labels == 0 {
            return Err(BadInput("a menu without a label list").into());
        }
        if !(1..=LABEL_LIMIT).any(|n| dword(labels + n * LABEL_LEN) == END) {
            return Err(BadInput("a label list without an end").into());
        }
        // Converted: `rc2k_core::screens::menus::labels` (the list read here: the first label
        // drawn without being checked for the end, as the original).
        use crate::screens::game_screen::{FontAt, MenuScreen};
        let mut list = Vec::new();
        let mut label = labels;
        loop {
            list.push(rc2k_core::screens::menus::Label {
                x: dword(label) as i32,
                y: dword(label + 4) as i32,
                font: FontAt(dword(label + 8)),
                id: dword(label + 0xC),
            });
            label += LABEL_LEN;
            if dword(label) == END {
                break;
            }
        }
        rc2k_core::screens::menus::labels(&mut MenuScreen::new(canvas), &list)
    }
}
