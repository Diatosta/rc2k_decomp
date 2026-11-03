//! `install::choice_by_number` (0x41486A): an item of choices (kind 2) set to choice ecx (a
//! number key's), if there is one and it is not marked.
//!
//! In:  esi the item, ecx the choice (`install::key_number`: the key less 2)
//! Out: edx the chosen byte's address, ebx the table; zero set, carry clear
//! The compare is of cl with the choices; the mark is read at `[ebx+ecx*2+1]`, all of ecx.
//! Callers `install::key_number`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::{self, MARKS};

/// `number` the choice (ecx, all 32 bits read for the mark).
pub fn choice_by_number(item: ListItem, number: u32) {
    let chosen_at = item.value();
    let table = item.choices();
    // Converted: `rc2k_core::install::items::choice_by_number`.
    let free = || table.marks(number) & MARKS == 0;
    if let Some(choice) = items::choice_by_number(number, item.count(), free) {
        chosen_at.set(choice);
    }
}
