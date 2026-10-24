//! `scene::clear_slot_textures` (0x4256A8): no texture slot holds or wants a texture.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `g_slotLoadedTextures` and `g_slotWantedTextures`, 0x400 dwords each, so that
//! every slot is loaded anew. One caller, `scene::reload_slot_textures`.

use crate::framework::{game_addr, rva};

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 2] = [
    (rva(0x5DB968), 0x1000, 0xFF), // g_slotLoadedTextures
    (rva(0x5DA968), 0x1000, 0xFF), // g_slotWantedTextures
];

// ---- Rust version ----

pub unsafe fn clear_slot_textures() {
    unsafe {
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
}
