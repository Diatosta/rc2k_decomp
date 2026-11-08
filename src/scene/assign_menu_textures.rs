//! `scene::assign_menu_textures` (0x490DF3): the eight slots of kind 0x258 get the
//! menu's own textures.
//!
//! For each of the eight numbers of `dword_75A622` that is not -1: the slot (kind 0x258's first
//! slot, `dword_5D2448`, plus its place in the eight) wants that number plus kind 0x258's first
//! texture (`dword_5D1448`). Two callers. Refuses a slot past the table.

use crate::errors::BadInput;
use crate::menu::owner::Tables;
use crate::session::read_all;

// ---- game data ----
const KIND: usize = 0x258;

// ---- Rust version ----

pub fn assign_menu_textures(
    tables: &Tables,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let textures = &scene.textures;
    // Converted: `rc2k_core::scene::textures::menu_textures`.
    let wanted = rc2k_core::scene::textures::menu_textures(
        read_all(&tables.lists.carousel_textures),
        textures.kinds.kind_first_slots[KIND].get(),
        textures.kinds.kind_first_textures[KIND].get(),
    )?;
    for (slot, texture) in wanted {
        crate::scene::texture_places_store::cell_at(&textures.places.slot_wanted[0], slot)
            .set(texture);
    }
    Ok(())
}
