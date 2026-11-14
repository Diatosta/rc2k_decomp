//! What the four redefine keys pages share ("REDEFINIR TECLADO", `defkeys.png`): the twelve
//! keys a player drives with (`Player::keys`, +0x1C), a copy of them the page works on, and the
//! key being redefined.
//!
//! An item's action (`menu::choose_key_0` .. `_11`) sets the redefining key to 1 and the
//! redefined key place to its key's place; then once a frame `redefine_key` takes the pending
//! key: none does nothing; Escape (1) gives the copy to the player unchanged and ends; any
//! other key (its low seven bits) swaps with the redefined place's old key if the copy already
//! has it, is not taken if it is in the reserved list (the page keeps waiting), else goes to
//! its place; then the place becomes 999, the copy goes to the player and the redefining ends.
//! Place 8 has no item on the pages.

use crate::cells::Store;
use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::owner::Keys;
use crate::menu::owner::Menus;

pub use rc2k_core::menu::player::{KEYS, Redefined};
pub const NO_PLACE: u32 = 999;
/// The reserved list's length is not known; the game's has 18 keys.
const RESERVED_ROOM: usize = 0x100;

/// The reserved keys, up to their NUL; `None` if there is none in `RESERVED_ROOM`.
fn reserved(keys: &Keys) -> Result<Option<Vec<u8>>, BadInput> {
    let store = &keys.tables.reserved_keys;
    let mut list = vec![0; store.size().min(RESERVED_ROOM)];
    store.load(0, &mut list)?;
    Ok(list
        .iter()
        .position(|&b| b == 0)
        .map(|end| list[..end].to_vec()))
}

/// One frame of redefining a key of `player` (0: refused).
pub fn redefine_key(
    menus: &Menus,
    input: &crate::input::owner::Input,
    player: Option<crate::menu::standings::PlayerCell>,
) -> Result<(), Refused> {
    if !menus.screens.entry.redefining_key.is_set() {
        return Ok(());
    }
    // Everything that could be refused, before the key is taken.
    let Some(player) = player else {
        return Err(BadInput("no player").into());
    };
    let place: u32 = menus.screens.entry.redefined_key_place.get();
    if place as usize >= KEYS {
        return Err(BadInput("a key's place of 12 or more").into());
    }
    let Some(reserved) = reserved(&menus.keys)? else {
        return Err(BadInput("a reserved list without its end").into());
    };
    let work: [u8; KEYS] = std::array::from_fn(|i| menus.keys.tables.redefined_keys.get(i));
    let Redefined::Done { keys, placed } =
        rc2k_core::menu::player::redefine(work, place as usize, take_key(input), &reserved)
    else {
        return Ok(());
    };
    for (i, &key) in keys.iter().enumerate() {
        menus.keys.tables.redefined_keys.set(i, key);
    }
    if placed {
        menus.screens.entry.redefined_key_place.set(NO_PLACE);
    }
    player.update(|p| p.keys = keys);
    menus.screens.entry.redefining_key.set(false);

    Ok(())
}
