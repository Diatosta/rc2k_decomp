//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords each, so that
//! every slot is loaded anew. One caller, `scene::reload_slot_textures`.

use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
fn fills() -> [(usize, usize, u8); 2] {
    [
        (TEXTURE_PLACES.slot_loaded[0].addr(), 0x1000, 0xFF), // g_slotLoadedTextures
        (TEXTURE_PLACES.slot_wanted[0].addr(), 0x1000, 0xFF), // g_slotWantedTextures
    ]
}

// ---- Rust version ----

pub unsafe fn clear_slot_textures() {
    unsafe {
        for (table, len, byte) in fills() {
            std::ptr::write_bytes(table as *mut u8, byte, len);
        }
    }
}
