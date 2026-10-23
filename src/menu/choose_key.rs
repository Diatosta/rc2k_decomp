//! What the twelve `menu::choose_key_0` .. `_11` (0x4682D3 .. 0x4683BA, 0x15 bytes apart)
//! share: the actions of the redefine keys pages' items, each starting the redefining of
//! its key's place (`menu/redefine_keys.rs`): `g_redefiningKey` = 1 and
//! `g_redefinedKeyPlace` = the place. `choose_key_8` is in no menu.

use crate::framework::game_write;
use crate::menu::redefine_keys::{PLACE_RVA, REDEFINING_RVA};

pub fn choose_key(place: u32) {
    // Converted: `rc2k_core::menu::player::Redefining`.
    let r = rc2k_core::menu::player::Redefining::start(place);
    unsafe {
        game_write(REDEFINING_RVA, r.on);
        game_write(PLACE_RVA, r.place);
    }
}
