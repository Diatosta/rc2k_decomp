//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! In:  nothing
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original clears the table
//! first; the result is the same.) One caller.

use crate::framework::{game_addr, game_write, rva};

// ---- game data ----

/// What is filled, in this order: where, how many bytes, with which byte.
const FILLS: [(usize, usize, u8); 1] = [
    (rva(0x63661C), 0x1000, 0x00), // g_fogTable
];

/// Cleared first.
const CLEARED_RVA: usize = rva(0x636610); // dword_636610

// ---- Rust version ----

pub unsafe fn clear_fog() {
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        for (table, len, byte) in FILLS {
            std::ptr::write_bytes(game_addr(table) as *mut u8, byte, len);
        }
    }
}
