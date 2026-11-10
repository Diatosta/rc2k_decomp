//! `scene::forget_slot_texture` (0x44EB66): slot eax's loaded texture forgotten
//! (`g_slotLoadedTextures[eax]` -1), so that it is loaded again.
//!
//! Takes: the slot
//!
//! 12 callers (among them the marks' snow textures). Refuses a slot past 0x3FF (the table is
//! 0x400 dwords).

use crate::errors::Refused;

pub fn forget_slot_texture(slot: u32, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    // Converted: `rc2k_core::scene::textures::forget_slot_texture`.
    Ok(crate::scene::textures::with_cells(
        [&scene.textures.places.slot_loaded],
        |[loaded]| rc2k_core::scene::textures::forget_slot_texture(loaded, slot),
    )?)
}
