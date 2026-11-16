//! An item of choices (kind 2) moved to the next choice that is not marked (bits 6, 7 of its second
//! byte), wrapping from the last to the first; none: nothing changes. Refused: a chosen byte past
//! the choices with every choice marked.

use crate::errors::Refused;
use crate::install::choice_back::{choice_places, store_choice};
use crate::memory::list_handle::ListItem;

pub fn choice_on(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<(), Refused> {
    let (chosen_at, free) = choice_places(install, item);
    let chosen = chosen_at.get();
    let cl = rc2k_core::install::items::choice_on(chosen, || item.count(), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
