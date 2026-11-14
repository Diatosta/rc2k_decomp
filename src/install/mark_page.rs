//! A page's items marked for drawing, item by item (a count of 0 is 256):
//! - a choice item (kind 2) marked 0x80 (not to be chosen) gives every unmarked choice 0x80
//!   too; one unmarked with at most one free choice has its chosen byte become that one (with
//!   none, the choices' count less 2) and the item 0x80;
//! - the highlighted item, if unmarked: an edit (kind 4) whose record has bit 2 is begun
//!   (`install::next_edit`), and an edit being edited, or any item while the blink clock's bit 3
//!   is clear, gets 0xC0 (lit);
//! - a choice item's chosen choice gets 0xC0.

use crate::menu::list_handle::{ListItem, ListMenu};
use rc2k_core::install::items::{self, MARKS};
use rc2k_core::install::pages::UNUSABLE;

pub fn mark_page(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    list: ListMenu,
) {
    let table = |item: ListItem| {
        item.choices()
            .bytes(2 * items::walked(item.count()), "an item's choices")
            .unwrap_or_else(|e| crate::install::installer::refuse(e))
    };
    let item_count = match list.count() {
        0 => 256,
        n => n as u32,
    };
    for index in 0..item_count {
        let item = list.item(index);
        let byte = item.first();
        if byte & 0xF == 2 {
            let marks = byte & MARKS;
            if marks == UNUSABLE {
                let mut choices = table(item);
                items::mark_choices(&mut choices, marks);
                item.choices()
                    .write_bytes(&choices, "an item's choices")
                    .unwrap_or_else(|e| crate::install::installer::refuse(e));
            } else if marks == 0
                && let Some(chosen) = items::lone_choice(&table(item), item.count())
            {
                item.value().set(chosen);
                item.set_first(item.first() | UNUSABLE);
            }
        }
        let byte = item.first();
        if byte & MARKS == 0 && index == list_state.choice.get() {
            let mut editing = None;
            if byte & 0xF == 4 {
                let record = item.edit_field();
                if record.flags() & 4 != 0 {
                    crate::install::next_edit::next_edit(install, list_state, 0, list);
                }
                editing = Some(record.flags() & 1 != 0);
            }
            let clock = || {
                list_state
                    .frame_clock
                    .get()
                    .wrapping_sub(list_state.move_clock.get())
            };
            if editing == Some(true) || items::lit(editing, clock()) {
                item.set_first(item.first() | MARKS);
            }
        }
        if byte & 0xF == 2 {
            let chosen = item.value().get();
            if chosen < item.count() {
                let choices = item.choices();
                choices.set_marks(chosen as u32, choices.marks(chosen as u32) | MARKS);
            }
        }
    }
}
