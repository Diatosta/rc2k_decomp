//! `scene::assign_menu_textures` (0x490DF3): the eight slots of kind 0x258 get the
//! menu's own textures.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! For each of the eight numbers of `dword_75A622` that is not -1: the slot (kind 0x258's first
//! slot, `dword_5D2448`, plus its place in the eight) wants that number plus kind 0x258's first
//! texture (`dword_5D1448`). Two callers. Refuses a slot past the table.

use crate::errors::BadInput;
use crate::menu::owner::menus;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::texture_places_store::TEXTURE_PLACES;
use crate::session::read_all;

// ---- game data ----
const KIND: usize = 0x258;

// ---- Rust version ----

pub fn assign_menu_textures() -> Result<(), BadInput> {
    // Converted: `rc2k_core::scene::textures::menu_textures`.
    let wanted = rc2k_core::scene::textures::menu_textures(
        read_all(&menus().tables.lists.carousel_textures),
        KIND_TEXTURES.kind_first_slots[KIND].get(),
        KIND_TEXTURES.kind_first_textures[KIND].get(),
    )?;
    for (slot, texture) in wanted {
        crate::scene::texture_places_store::cell_at(&TEXTURE_PLACES.slot_wanted[0], slot)
            .set(texture);
    }
    Ok(())
}
