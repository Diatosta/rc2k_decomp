//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `dword_5D9968` and `dword_5D8968` (each texture's place), 0x400 dwords each.
//! Two callers.

// ---- game data ----

// ---- Rust version ----

pub fn reset_texture_places() {
    // dword_5D9968, dword_5D8968: 0x1000 bytes each, 0xFF.
    for table in [
        &crate::scene::owner::scene().textures.places.loaded_pictures,
        &crate::scene::owner::scene().textures.places.wanted_pictures,
    ] {
        for cell in table {
            cell.set(u32::MAX);
        }
    }
}
