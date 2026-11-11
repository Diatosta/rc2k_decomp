//! What the four redefine keys pages share ("REDEFINIR TECLADO", `defkeys.png`): the
//! twelve keys a player drives with (`Player::keys`, +0x1C), a copy of them the page works
//! on (`g_redefinedKeys`, 0x755595, filled by the page's set-up), and the key being
//! redefined.
//!
//! An item's action (`menu::choose_key_0` .. `_11`, 0x4682D3 ..) sets `g_redefiningKey`
//! (0x7556AC) to 1 and `g_redefinedKeyPlace` (0x7556B0) to its key's place; then once a
//! frame `redefine_key` takes the pending key:
//!   none          nothing;
//!   1 (Escape)    the copy goes to the player unchanged and the redefining ends;
//!   any other     its low seven bits; a key the copy already has gets the redefined
//!                 place's old key (the two swap); otherwise a key of the reserved list
//!                 (`g_reservedKeys`, 0x755621, ended by a NUL) is not taken and the page
//!                 goes on waiting; then the key goes to its place, the place becomes 999
//!                 (0x3E7), the copy goes to the player and the redefining ends.
//! The place 8 has no item on the pages (it is not drawn either).

use crate::errors::{BadInput, Refused};
use crate::input::take_key::take_key;
use crate::menu::owner::Keys;
use crate::menu::owner::Menus;
use crate::pointers::Store;

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
    player: Option<crate::menu::standings::PlayerCell>,
) -> Result<(), Refused> {
    if !menus.screens.option_work.redefining_key.is_set() {
        return Ok(());
    }
    // Everything that could be refused, before the key is taken.
    let Some(player) = player else {
        return Err(BadInput("no player").into());
    };
    let place: u32 = menus.screens.option_work.redefined_key_place.get();
    if place as usize >= KEYS {
        return Err(BadInput("a key's place of 12 or more").into());
    }
    let Some(reserved) = reserved(&menus.keys)? else {
        return Err(BadInput("a reserved list without its end").into());
    };
    // The logic is in `rc2k_core::menu::player::redefine`.
    let work: [u8; KEYS] = std::array::from_fn(|i| menus.keys.tables.redefined_keys.get(i));
    let Redefined::Done { keys, placed } =
        rc2k_core::menu::player::redefine(work, place as usize, take_key(), &reserved)
    else {
        return Ok(());
    };
    for (i, &key) in keys.iter().enumerate() {
        menus.keys.tables.redefined_keys.set(i, key);
    }
    if placed {
        menus.screens.option_work.redefined_key_place.set(NO_PLACE);
    }
    player.update(|p| p.keys = keys);
    menus.screens.option_work.redefining_key.set(false);

    Ok(())
}
