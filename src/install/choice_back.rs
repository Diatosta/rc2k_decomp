//! An item of choices (kind 2) moved to the previous choice that is not marked (bits 6, 7 of
//! its second byte), wrapping from the first to the last; none: nothing changes. Refused: a
//! chosen byte past the choices with every choice marked. Called by `install::key_up`,
//! `install::key_left`.

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
    let cl = items::choice_back(chosen, || item.count(), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
