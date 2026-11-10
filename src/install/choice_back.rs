//! `install::choice_back` (0x414826): an item of choices (kind 2) moved to the choice before
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the first
//! to the last; none: nothing changes.
//!
//! The step down is `dec cl; jns`: from 0x80 .. 0xFF (and 0) it goes to the last choice.
//! A chosen byte past the choices with every choice marked never ends: refused (the
//! original goes round for ever). Callers `install::key_up`, `install::key_left`.

use crate::errors::Refused;
use crate::menu::list_handle::{ItemByte, ListItem};
use rc2k_core::install::items::{self, MARKS};

/// An item of choices' chosen byte; and whether choice `cl` is free.
pub fn choice_places(item: ListItem) -> (ItemByte, impl Fn(u8) -> bool + use<>) {
    let table = item.choices();
    let free = move |cl: u8| table.marks(cl as u32) & MARKS == 0;
    (item.value(), free)
}

/// The choice reached stored, when it changed.
pub fn store_choice(chosen_at: ItemByte, chosen: u8, cl: u8) {
    if cl != chosen {
        chosen_at.set(cl);
    }
}

pub fn choice_back(item: ListItem) -> Result<(), Refused> {
    let (chosen_at, free) = choice_places(item);
    let chosen = chosen_at.get();
    // Converted: `rc2k_core::install::items::choice_back`.
    let cl = items::choice_back(chosen, || item.count(), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
