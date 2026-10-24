//! `scene::reset_texture_places` (0x423FE1): no texture has a place.
//!
//! In:  nothing
//! Out: eax = 0xffffffff, ecx = 0, edi = the end of the last table
//!
//! -1 throughout `dword_5D9968` and `dword_5D8968` (each texture's place), 0x400 dwords each.
//! Two callers.

use crate::framework::{game_addr, rva};

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 2] = [
    (rva(0x5D9968), 0x1000, 0xFF), // dword_5D9968
    (rva(0x5D8968), 0x1000, 0xFF), // dword_5D8968
];

// ---- Rust version ----

pub unsafe fn reset_texture_places() {
    unsafe {
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
}
