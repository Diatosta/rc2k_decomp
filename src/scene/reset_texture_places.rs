//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `dword_5D9968` and `dword_5D8968` (each texture's place), 0x400 dwords each.
//! Two callers.

use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
fn fills() -> [(usize, usize, u8); 2] {
    [
        (TEXTURE_PLACES.loaded_pictures[0].addr(), 0x1000, 0xFF), // dword_5D9968
        (TEXTURE_PLACES.wanted_pictures[0].addr(), 0x1000, 0xFF), // dword_5D8968
    ]
}

// ---- Rust version ----

pub unsafe fn reset_texture_places() {
    unsafe {
        for (table, len, byte) in fills() {
            std::ptr::write_bytes(table as *mut u8, byte, len);
        }
    }
}
