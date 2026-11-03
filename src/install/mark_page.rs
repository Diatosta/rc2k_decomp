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

use crate::install::vars::INSTALLER_PAGES;
use crate::menu::list_handle::{ListItem, ListMenu};
use rc2k_core::install::items::{self, MARKS};
use rc2k_core::install::pages::UNUSABLE;

pub fn mark_page(list: ListMenu) {
    let mut cl = 0u8;
    // Converted: `rc2k_core::install::items` (`walked`, `mark_choices`, `lone_choice`,
    // `lit`).
    let table = |item: ListItem| {
        item.choices()
            .bytes(2 * items::walked(item.count()), "an item's choices")
            .unwrap_or_else(|e| crate::install::installer::refuse(e))
    };
    loop {
        let esi = list.item(cl as u32);
        let byte = esi.first();
        if byte & 0xF == 2 {
            let ah = byte & MARKS;
            if ah == UNUSABLE {
                let mut choices = table(esi);
                items::mark_choices(&mut choices, ah);
                esi.choices()
                    .write_bytes(&choices, "an item's choices")
                    .unwrap_or_else(|e| crate::install::installer::refuse(e));
            } else if ah == 0
                && let Some(chosen) = items::lone_choice(&table(esi), esi.count())
            {
                esi.value().set(chosen);
                esi.set_first(esi.first() | UNUSABLE);
            }
        }
        let byte = esi.first();
        if byte & MARKS == 0 && cl as u32 == INSTALLER_PAGES.choice.get() {
            let mut editing = None;
            if byte & 0xF == 4 {
                let record = esi.edit_field();
                if record.flags() & 4 != 0 {
                    crate::install::next_edit::next_edit(0, list);
                }
                editing = Some(record.flags() & 1 != 0);
            }
            let clock = || {
                INSTALLER_PAGES
                    .frame_clock
                    .get()
                    .wrapping_sub(INSTALLER_PAGES.move_clock.get())
            };
            if editing == Some(true) || items::lit(editing, clock()) {
                esi.set_first(esi.first() | MARKS);
            }
        }
        if byte & 0xF == 2 {
            let chosen = esi.value().get();
            if chosen < esi.count() {
                let choices = esi.choices();
                choices.set_marks(chosen as u32, choices.marks(chosen as u32) | MARKS);
            }
        }
        cl = cl.wrapping_add(1);
        if cl == list.count() {
            break;
        }
    }
}
