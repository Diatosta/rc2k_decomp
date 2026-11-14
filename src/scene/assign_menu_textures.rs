//! `scene::assign_menu_textures`: the eight slots of kind 0x258 get the menu's own textures.
//!
//! For each of the eight numbers of `MenuLists.carousel_textures` that is not -1: the slot
//! (kind 0x258's first slot, `KindTextureTables.kind_first_slots[600]`, plus its place in the
//! eight) wants that number plus kind 0x258's first texture
//! (`KindTextureTables.kind_first_textures[600]`). Two callers. Refuses a slot past the table.

use crate::errors::BadInput;
use crate::menu::lists_store::lists;
use crate::session::read_all;

// ---- game data ----
const KIND: usize = 0x258;

pub fn assign_menu_textures(scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    let textures = &scene.textures;
    // Logic: `rc2k_core::scene::textures::menu_textures`.
    let wanted = rc2k_core::scene::textures::menu_textures(
        read_all(&lists().carousel_textures),
        textures.kinds.get().kind_first_slots[KIND].get(),
        textures.kinds.get().kind_first_textures[KIND].get(),
    )?;
    for (slot, texture) in wanted {
        crate::scene::texture_places_store::cell_at(
            textures.places.get(),
            crate::scene::texture_places_store::PlacesTable::SlotWanted,
            slot,
        )?
        .set(texture);
    }
    Ok(())
}
