//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords each, so that
//! every slot is loaded anew. One caller, `scene::reload_slot_textures`.

use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

// ---- Rust version ----

pub fn clear_slot_textures() {
    // g_slotLoadedTextures, g_slotWantedTextures: 0x1000 bytes each, 0xFF.
    for table in [&TEXTURE_PLACES.slot_loaded, &TEXTURE_PLACES.slot_wanted] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
