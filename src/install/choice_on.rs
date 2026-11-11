//! `install::choice_on` (0x414848): an item of choices (kind 2) moved to the choice after
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the last to
//! the first; none: nothing changes.
//!
//! The step up goes to the next byte while it is below the choices' count (else 0). A chosen byte past the choices
//! with every choice marked never ends: refused (the original goes round for ever).
//! Callers `install::key_down`, `install::key_right`.

use crate::errors::Refused;
use crate::install::choice_back::{choice_places, store_choice};
use crate::menu::list_handle::ListItem;

pub fn choice_on(item: ListItem) -> Result<(), Refused> {
    let (chosen_at, free) = choice_places(item);
    let chosen = chosen_at.get();
    // Converted: `rc2k_core::install::items::choice_on`.
    let cl = rc2k_core::install::items::choice_on(chosen, || item.count(), free)?;
    store_choice(chosen_at, chosen, cl);
    Ok(())
}
