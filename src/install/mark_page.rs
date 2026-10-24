//! `install::mark_page` (0x414B53): a page's items marked for drawing, item by item (cl
//! counts them, `[edi+1]` of them, 256 for 0):
//! - a choice item (kind 2) marked 0x80 (not to be chosen) gives every unmarked choice
//!   0x80 too; one unmarked has its free choices counted, and with at most one its chosen
//!   byte becomes that one (`neg al; add al, [esi+1]`: with none, the choices less 2) and
//!   the item 0x80;
//! - the highlighted item (`g_listChoice`), if unmarked: an edit (kind 4) whose record has
//!   bit 2 is begun (`install::next_edit`, ebx 0), and an edit being edited, or any item
//!   while the clock's bit 3 (`dword_51D41F - dword_51D423`) is clear, gets 0xC0 (lit);
//! - a choice item's chosen choice, if it is one, gets 0xC0.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//! Callers: `install::run_page` (twice).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{CHOICE, FRAME_CLOCK, MOVE_CLOCK, g, rb, rd, wb};
use rc2k_core::install::items::{self, MARKS};
use rc2k_core::install::pages::UNUSABLE;

pub fn mark_page(list: u32) {
    let mut cl = 0u8;
    let mut esi = list + 0x18;
    // Converted: `rc2k_core::install::items` (`walked`, `mark_choices`, `lone_choice`,
    // `lit`).
    let table = |item: u32| unsafe {
        std::slice::from_raw_parts_mut(rd(item + 0xC) as *mut u8, 2 * items::walked(rb(item + 1)))
    };
    loop {
        let byte = rb(esi);
        if byte & 0xF == 2 {
            let ah = byte & MARKS;
            if ah == UNUSABLE {
                items::mark_choices(table(esi), ah);
            } else if ah == 0
                && let Some(chosen) = items::lone_choice(table(esi), rb(esi + 1))
            {
                wb(rd(esi + 8), chosen);
                wb(esi, rb(esi) | UNUSABLE);
            }
        }
        let byte = rb(esi);
        if byte & MARKS == 0 && cl as u32 == g(CHOICE) {
            let mut editing = None;
            if byte & 0xF == 4 {
                let record = rd(esi + 0xC);
                if rb(record) & 4 != 0 {
                    crate::install::next_edit::next_edit(0, list);
                }
                editing = Some(rb(record) & 1 != 0);
            }
            let clock = || g(FRAME_CLOCK).wrapping_sub(g(MOVE_CLOCK));
            if editing == Some(true) || items::lit(editing, clock()) {
                wb(esi, rb(esi) | MARKS);
            }
        }
        if byte & 0xF == 2 {
            let chosen = rb(rd(esi + 8));
            if chosen < rb(esi + 1) {
                let at = rd(esi + 0xC)
                    .wrapping_add(1)
                    .wrapping_add(2 * chosen as u32);
                wb(at, rb(at) | MARKS);
            }
        }
        esi = esi.wrapping_add(0x10);
        cl = cl.wrapping_add(1);
        if cl == rb(list + 1) {
            break;
        }
    }
}
