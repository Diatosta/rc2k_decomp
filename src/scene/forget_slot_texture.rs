//! `scene::forget_slot_texture` (0x44EB66): slot eax's loaded texture forgotten
//! (`g_slotLoadedTextures[eax]` -1), so that it is loaded again.
//!
//! In:  eax the slot
//! Out: every register kept
//!
//! 12 callers (among them the marks' snow textures). Refuses a slot past 0x3FF (the table is
//! 0x400 dwords).

use crate::errors::Refused;
use crate::scene::texture_places_store::TEXTURE_PLACES;

/// The Rust version (see the header).
pub unsafe fn forget_slot_texture(slot: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::scene::textures::forget_slot_texture`.
    let loaded = unsafe { crate::scene::textures::cells_mut(&TEXTURE_PLACES.slot_loaded) };
    Ok(rc2k_core::scene::textures::forget_slot_texture(
        loaded, slot,
    )?)
}
