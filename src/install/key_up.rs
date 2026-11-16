//! Key 0x66 on an item: for choices shown upright (kind 0x22) the previous choice
//! (`install::choice_back`).

use crate::errors::Refused;
use crate::memory::list_handle::ListItem;
use rc2k_core::install::items::{Helper, ItemKey};

/// A key on `item`: the helper the key has for the item's kind, `number` for the number
/// keys' helpers; whether the key was taken.
pub fn item_key(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
    key: ItemKey,
    number: u32,
) -> Result<bool, Refused> {
    let kind = item.first() & key.kind_mask();
    use crate::install as i;
    match key.helper(kind) {
        Some(Helper::ChoiceBack) => i::choice_back::choice_back(install, item)?,
        Some(Helper::ChoiceOn) => i::choice_on::choice_on(install, item)?,
        Some(Helper::ChoiceByNumber) => {
            i::choice_by_number::choice_by_number(install, item, number);
        }
        Some(Helper::ValueDown) => i::value_down::value_down(install, list_state, item),
        Some(Helper::ValueUp) => i::value_up::value_up(install, list_state, item),
        Some(Helper::ValueByNumber) => i::value_by_number::value_by_number(install, item, number)?,
        None => return Ok(kind == key.last_compared()),
    }
    Ok(true)
}

/// Whether the key was taken.
pub fn key_up(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) -> Result<bool, Refused> {
    item_key(install, list_state, item, ItemKey::Up, 0)
}
