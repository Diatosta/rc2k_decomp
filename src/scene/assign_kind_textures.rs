//! `scene::assign_kind_textures` (0x4256D8): every texture slot gets the texture of
//! the kind it is the first slot of.
//!
//! The 0x400 slots of `TexturePlaces.slot_wanted[0]` = -1; then for the kinds from 0x3FF down to 0: the kind's
//! first slot (`KindTextureTables.kind_first_slots[0]`), unless it is -1, gets the kind's first texture
//! (`KindTextureTables.kind_first_textures[0]`). So of two kinds with one slot the lower one's stays. Six callers. Refuses
//! a first slot past the table.

use crate::errors::BadInput;
use crate::scene::textures::with_cells;

// ---- Rust version ----

pub fn assign_kind_textures(scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    let scene_textures = &scene.textures;
    // Converted: `rc2k_core::scene::textures::assign_kind_textures`.
    with_cells(
        [
            &scene_textures.kinds.get().kind_first_slots,
            &scene_textures.kinds.get().kind_first_textures,
            &scene_textures.places.get().slot_wanted,
        ],
        |[slots, textures, wanted]| {
            rc2k_core::scene::textures::assign_kind_textures(slots, textures, wanted)
        },
    )?;
    Ok(())
}
