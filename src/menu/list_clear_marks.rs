//! `menu::list_clear_marks`: the marks (bits 6 and 7) of a list menu's items cleared: each of
//! its items (16 bytes from +0x18) has its first byte masked with 0x3F, and for an item of
//! kind 2 (low nibble) its choices (2 bytes each from the choice table's second byte) too.
//!
//! A list of no items goes round 2^32 times, and a kind 2 of no choices 256 times (the lists passed
//! have items). The installer's lists pass the installer; the replay viewer's pass none (its
//! items have no choices, which are refused).

use crate::memory::list_handle::ListMenu;
use rc2k_core::menu::list_menu::{has_choices, rounds, unmarked};

pub fn list_clear_marks(install: Option<&'static crate::install::owner::Install>, list: ListMenu) {
    // Its count is a byte, but it is counted down as a dword: 0 goes round 2^32 times.
    let mut count = list.count() as u32;
    let mut n = 0;
    loop {
        let item = list.item(n);
        let kind = unmarked(item.first());
        item.set_first(kind);
        if has_choices(kind) {
            let Some(install) = install else {
                crate::fatal::raise::end(
                    "menu::list_clear_marks",
                    crate::errors::BadInput("an item with choices in a list that has no installer"),
                )
            };
            let choices = item.choices(install);
            for c in 0..rounds(item.count()) as u32 {
                choices.set_marks(c, unmarked(choices.marks(c)));
            }
        }
        n = n.wrapping_add(1);
        count = count.wrapping_sub(1);
        if count == 0 {
            break;
        }
    }
}
