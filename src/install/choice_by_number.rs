//! An item of choices (kind 2) set to choice `number` (a number key's), if there is one and it
//! is not marked. Called by `install::key_number`.

use crate::memory::list_handle::ListItem;
use rc2k_core::install::items::{self, MARKS};

/// `number` the choice (all 32 bits read for the mark).
pub fn choice_by_number(
    install: &'static crate::install::owner::Install,
    item: ListItem,
    number: u32,
) {
    let chosen_at = item.value(install);
    let table = item.choices(install);
    let free = || table.marks(number) & MARKS == 0;
    if let Some(choice) = items::choice_by_number(number, item.count(), free) {
        chosen_at.set(choice);
    }
}
